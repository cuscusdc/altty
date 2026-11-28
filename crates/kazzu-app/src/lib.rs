//! Kazzu: interface Slint do player de música.
//!
//! No Android a LIB (cdylib do NativeActivity) usa apenas `ui` (interface mobile v6) e `android`.
//! No desktop a árvore completa de controladores/view-models continua disponível.

pub mod ui {
    slint::include_modules!();
}

#[cfg(target_os = "android")]
pub mod android;

#[cfg(not(target_os = "android"))]
pub mod accent;
#[cfg(not(target_os = "android"))]
mod accounts;
#[cfg(not(target_os = "android"))]
pub mod app;
#[cfg(not(target_os = "android"))]
pub mod art;
#[cfg(not(target_os = "android"))]
pub mod catalog;
#[cfg(not(target_os = "android"))]
pub mod config;
#[cfg(not(target_os = "android"))]
pub mod controller;
#[cfg(not(target_os = "android"))]
pub mod covers;
#[cfg(all(not(target_os = "android"), feature = "demo"))]
pub mod demo;
#[cfg(not(target_os = "android"))]
pub mod format;
#[cfg(not(target_os = "android"))]
pub mod lru;
#[cfg(not(target_os = "android"))]
pub mod platform;
#[cfg(not(target_os = "android"))]
pub mod slint_view;
#[cfg(not(target_os = "android"))]
pub mod snapshot;
#[cfg(not(target_os = "android"))]
pub mod view;
#[cfg(not(target_os = "android"))]
pub mod vm;
#[cfg(not(target_os = "android"))]
pub mod remote;
#[cfg(not(target_os = "android"))]
pub mod worker;
#[cfg(not(target_os = "android"))]
pub mod ytm_vm;
