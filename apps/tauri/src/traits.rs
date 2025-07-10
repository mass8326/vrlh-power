use tauri::AppHandle;
use vrlh_power_manager_core::{Device, DeviceInfo, DeviceLocalStatus, DeviceRemoteStatus};

use crate::events::EventEmitter;

pub trait EmitAppDeviceEvent {
    async fn emit(self, app: &AppHandle, device: &Device) -> crate::Result<()>;
}

impl EmitAppDeviceEvent for DeviceLocalStatus {
    async fn emit(self, app: &AppHandle, device: &Device) -> crate::Result<()> {
        let info = DeviceInfo::from_device_local_status(device, self);
        app.emit_device_update(info)
    }
}

impl EmitAppDeviceEvent for DeviceRemoteStatus {
    async fn emit(self, app: &AppHandle, device: &Device) -> crate::Result<()> {
        let info = DeviceInfo::from_device_remote_status(device, self);
        app.emit_device_update(info)
    }
}
