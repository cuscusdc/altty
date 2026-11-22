//! Ponto de entrada e integração com o Android (NativeActivity via `android-activity`, backend do Slint).
//!
//! Só compila com `target_os = "android"` (ver `lib.rs`). Esta primeira versão:
//!
//! - guarda banco e configuração em `files/` do app (armazenamento interno, sem permissão);
//! - lê as músicas da pasta `Music` do armazenamento compartilhado, com a permissão `READ_MEDIA_AUDIO`
//!   (Android 13+) ou `READ_EXTERNAL_STORAGE` (até o Android 12), pedida em tempo de execução via JNI;
//! - não tem seletor de pastas (o botão "Adicionar pasta" adiciona/reescaneia `Music`). O seletor do sistema
//!   (SAF) devolve URIs de conteúdo, não caminhos, e exigiria reescrever o scan; fica para uma próxima etapa.
//!
//! Nenhum serviço do Google, nenhuma telemetria: só `INTERNET` (para o YouTube Music, que chega em breve),
//! o acesso às músicas e o áudio. Ver `docs/ANDROID.md`.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::sync::OnceLock;

use jni::JNIEnv;
use jni::objects::{JObject, JValue};
use slint::android::AndroidApp;
use slint::android::android_activity::{MainEvent, PollEvent};

use crate::app::{App, AppError, Options};
use crate::controller::Controller;
use crate::slint_view::SlintView;

static APP: OnceLock<AndroidApp> = OnceLock::new();

thread_local! {
    /// Controller da janela (thread do `android_main`), para reagir à permissão concedida.
    static CONTROLLER: RefCell<Option<Weak<RefCell<Controller<SlintView>>>>> = const { RefCell::new(None) };
    /// A pasta de músicas já foi entregue à biblioteca nesta execução.
    static MUSIC_ADDED: RefCell<bool> = const { RefCell::new(false) };
}

/// Entrada do app: chamada pelo glue do `android-activity` na thread principal do app (não é a thread de UI do Java).
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("kazzu"));
    let _ = APP.set(app.clone());
    if let Err(e) = slint::android::init_with_event_listener(app.clone(), on_event) {
        log::error!("não foi possível iniciar o backend Android do Slint: {e}");
        return;
    }
    if let Err(e) = run(&app) {
        log::error!("kazzu: {e}");
    }
}

fn run(app: &AndroidApp) -> Result<(), AppError> {
    let base = app.internal_data_path().ok_or(AppError::NoDataDir)?;
    std::fs::create_dir_all(&base)?;
    // Skia (único renderer do backend Android) tem `backdrop-blur`; o modo software não é usado aqui.
    let opts = Options { data_dir: Some(base.join("data")), config_path: Some(base.join("config.toml")), ..Options::default() };
    let kazzu = App::build(&opts)?;
    CONTROLLER.with(|c| *c.borrow_mut() = Some(Rc::downgrade(&kazzu.controller)));
    ensure_music_folder();
    kazzu.run()
}

fn on_event(event: &PollEvent<'_>) {
    // Ao voltar do diálogo de permissão (ou de qualquer outra tela), tenta de novo.
    if let PollEvent::Main(MainEvent::Resume { .. }) = event {
        ensure_music_folder();
    }
}

/// Pasta `Music` do armazenamento compartilhado (ex.: `/storage/emulated/0/Music`).
pub fn music_dir() -> PathBuf {
    let root = APP
        .get()
        .and_then(|a| a.external_data_path())
        .and_then(|p| p.ancestors().find(|a| a.file_name().is_some_and(|n| n == "Android")).and_then(|a| a.parent()).map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("/storage/emulated/0"));
    root.join("Music")
}

/// Se há permissão de leitura de áudio, entrega `Music` à biblioteca (uma vez); senão, pede a permissão.
/// Devolve se a pasta já está (ou acabou de ser) entregue.
pub fn ensure_music_folder() -> bool {
    if MUSIC_ADDED.with(|m| *m.borrow()) {
        return true;
    }
    if !has_audio_permission() {
        request_audio_permission();
        return false;
    }
    MUSIC_ADDED.with(|m| *m.borrow_mut() = true);
    add_music_folder();
    true
}

/// Botão "Adicionar pasta": reentrega `Music` (a biblioteca ignora duplicatas e reescaneia) ou pede a permissão.
pub fn add_music_folder_from_ui() -> bool {
    if !has_audio_permission() {
        request_audio_permission();
        return false;
    }
    MUSIC_ADDED.with(|m| *m.borrow_mut() = true);
    add_music_folder();
    true
}

fn add_music_folder() {
    // Adia para o laço de eventos: o controller pode estar emprestado neste momento (ex.: dentro de um callback).
    let _ = slint::invoke_from_event_loop(|| {
        CONTROLLER.with(|c| {
            if let Some(rc) = c.borrow().as_ref().and_then(Weak::upgrade)
                && let Ok(mut ctrl) = rc.try_borrow_mut()
            {
                ctrl.add_folder_path(music_dir());
            }
        });
    });
}

// ---------------------------------------------------------------------------------------------
// Permissão (JNI)
// ---------------------------------------------------------------------------------------------

fn with_activity<R>(f: impl FnOnce(&mut JNIEnv, &JObject) -> jni::errors::Result<R>) -> Option<R> {
    let app = APP.get()?;
    // SAFETY: ponteiros da JavaVM e da Activity vivem enquanto o app roda (garantia do android-activity).
    let result = unsafe {
        let vm = jni::JavaVM::from_raw(app.vm_as_ptr().cast()).ok()?;
        let mut env = vm.attach_current_thread().ok()?;
        let activity = JObject::from_raw(app.activity_as_ptr().cast());
        let r = f(&mut env, &activity);
        // a referência da Activity pertence ao android-activity: não pode ser liberada aqui
        std::mem::forget(activity);
        r
    };
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            log::warn!("chamada JNI falhou: {e}");
            None
        }
    }
}

fn sdk_int(env: &mut JNIEnv) -> jni::errors::Result<i32> {
    env.get_static_field("android/os/Build$VERSION", "SDK_INT", "I")?.i()
}

fn audio_permission_name(env: &mut JNIEnv) -> jni::errors::Result<&'static str> {
    Ok(if sdk_int(env)? >= 33 { "android.permission.READ_MEDIA_AUDIO" } else { "android.permission.READ_EXTERNAL_STORAGE" })
}

fn has_audio_permission() -> bool {
    with_activity(|env, activity| {
        let permission = audio_permission_name(env)?;
        let name = env.new_string(permission)?;
        let granted = env.call_method(activity, "checkSelfPermission", "(Ljava/lang/String;)I", &[JValue::Object(&name)])?.i()?;
        Ok(granted == 0) // PackageManager.PERMISSION_GRANTED
    })
    .unwrap_or(false)
}

fn request_audio_permission() {
    let _ = with_activity(|env, activity| {
        let permission = audio_permission_name(env)?;
        let name = env.new_string(permission)?;
        let empty = env.new_string("")?;
        let array = env.new_object_array(1, "java/lang/String", &empty)?;
        env.set_object_array_element(&array, 0, &name)?;
        env.call_method(activity, "requestPermissions", "([Ljava/lang/String;I)V", &[JValue::Object(&array), JValue::Int(1)])?;
        Ok(())
    });
}
