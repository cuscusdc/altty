//! Kazzu: interface desktop (Slint) do player de música.
//!
//! - `controller`: lógica (motor de áudio + biblioteca + YouTube Music + tela), sem janela;
//! - `catalog`/`remote`/`covers`/`ytm_vm`: provedores remotos (rede fora da thread de UI) e seus view-models;
//! - `view`/`vm`: fronteira com a tela e dados prontos para ela;
//! - `slint_view`/`app`: ligação com o Slint (única parte que depende da janela);
//! - `worker`: biblioteca, scan e capas fora da thread de UI.

pub mod accent;
#[cfg(target_os = "android")]
pub mod android;
pub mod app;
pub mod art;
pub mod catalog;
pub mod config;
pub mod controller;
pub mod covers;
#[cfg(feature = "demo")]
pub mod demo;
pub mod format;
pub mod lru;
pub mod platform;
pub mod slint_view;
pub mod snapshot;
pub mod view;
pub mod vm;
pub mod remote;
pub mod worker;
pub mod ytm_vm;

/// Tipos gerados a partir de `ui/app.slint`.
pub mod ui {
    slint::include_modules!();
}
