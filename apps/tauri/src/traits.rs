use tauri::AppHandle;
use vrlh_power_manager_core::{Device, DeviceInfo, FromDeviceStatus};

use crate::events::EmitEvent;

pub trait EmitDeviceStatus<T> {
    fn emit_device(&self, device: &Device, status: T) -> crate::Result<()>;
}

impl<T> EmitDeviceStatus<T> for AppHandle
where
    DeviceInfo: FromDeviceStatus<T>,
{
    fn emit_device(&self, device: &Device, status: T) -> crate::Result<()> {
        let info = DeviceInfo::from_device_status(device, status);
        self.emit_event(info)
    }
}
