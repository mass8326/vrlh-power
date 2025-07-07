mod create_discovery_stream;
mod power_off;
mod power_on;

use std::{collections::HashMap, sync::Arc};

use btleplug::platform::{Adapter, Peripheral, PeripheralId};
use tokio::sync::Mutex;

use crate::get_default_adapter;

pub use create_discovery_stream::CreateDiscoveryStream;
pub use power_off::PowerOff;
pub use power_on::PowerOn;

#[derive(Clone, Debug, Default)]
pub struct DeviceList {
    pub map: Arc<Mutex<HashMap<PeripheralId, Peripheral>>>,
    pub adapter: Option<Adapter>,
}

impl DeviceList {
    pub async fn init(&mut self) -> crate::Result<()> {
        self.adapter = Some(get_default_adapter().await?);
        Ok(())
    }

    pub fn assert_adapter(&self) -> crate::Result<Adapter> {
        self.adapter
            .clone()
            .ok_or(crate::Error::Vrlh("Device list not initialized"))
    }
}
