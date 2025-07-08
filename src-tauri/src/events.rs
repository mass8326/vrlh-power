use serde::Serialize;
use tauri::{AppHandle, Emitter};
use vrlh_power_manager_core::DeviceUpdatePayload;

#[derive(Clone, Debug, Serialize)]
struct StatusPayload(String);

impl StatusPayload {
    pub fn new(msg: String) -> Self {
        return Self(msg);
    }
}

pub trait EventEmitter {
    fn emit_device_update(&self, payload: DeviceUpdatePayload) -> crate::Result<()>;
    fn emit_status(&self, payload: String) -> crate::Result<()>;
}

impl EventEmitter for AppHandle {
    fn emit_device_update(&self, payload: DeviceUpdatePayload) -> crate::Result<()> {
        self.emit("device-update", payload).map_err(Into::into)
    }

    fn emit_status(&self, msg: String) -> crate::Result<()> {
        self.emit("status", StatusPayload::new(msg))
            .map_err(Into::into)
    }
}
