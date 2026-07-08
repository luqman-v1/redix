pub mod client;
pub mod cluster;
pub mod sentinel;
pub mod standalone;
pub mod teleport;
pub mod tunnel;

pub use client::*;
pub use cluster::*;
pub use sentinel::*;
pub use standalone::*;
pub use teleport::*;
pub use tunnel::*;

pub fn emit_command_log(cmd: &str, duration_ms: u64) {
    use tauri::Emitter;
    if let Some(app) = crate::APP_HANDLE.get() {
        let payload = serde_json::json!({
            "command": cmd,
            "duration": duration_ms
        });
        let _ = app.emit("command-log", payload);
    }
}
