mod constants;
mod device;
mod device_list;
mod dto;
mod error;

use btleplug::{
    api::Manager as _,
    platform::{Adapter, Manager},
};

pub use device::Device;
pub use device_list::DeviceList;
pub use dto::*;
pub use error::*;

pub async fn get_default_adapter() -> crate::Result<Adapter> {
    Manager::new()
        .await
        .map_err(|_| crate::Error::Vrlh("Failed to create bluetooth session!"))?
        .adapters()
        .await
        .map_err(|_| crate::Error::Vrlh("Could not access bluetooth adapter!"))?
        .into_iter()
        .next()
        .ok_or(crate::Error::Vrlh("No bluetooth adapter available!"))
}
