use serde::Serialize;
use tauri::{AppHandle, Emitter};
use vrlh_power_manager_core::DeviceInfo;

#[derive(Clone, Debug, Serialize)]
pub struct StatusPayload(String);

impl StatusPayload {
    pub fn new(msg: String) -> Self {
        Self(msg)
    }
}

pub trait EmitEvent<T> {
    fn emit_event(&self, payload: T) -> crate::Result<()>;
}

impl EmitEvent<DeviceInfo> for AppHandle {
    fn emit_event(&self, payload: DeviceInfo) -> crate::Result<()> {
        self.emit("device-update", payload).map_err(Into::into)
    }
}

impl EmitEvent<StatusPayload> for AppHandle {
    fn emit_event(&self, payload: StatusPayload) -> crate::Result<()> {
        self.emit("status", payload).map_err(Into::into)
    }
}
