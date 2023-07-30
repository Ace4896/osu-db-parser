#[cfg(not(target_arch = "wasm32"))]
pub mod file_dialog;

#[cfg(target_arch = "wasm32")]
#[path = "widgets/file_dialog_wasm.rs"]
pub mod file_dialog;
