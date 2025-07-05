use btleplug::api::{Characteristic, Manager as _, Peripheral as _};
use btleplug::platform::{Adapter, Manager, Peripheral};

use crate::constants::{LHV2_GATT_POWER_CHARACTERISTIC, LHV2_GATT_POWER_SERVICE};

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

pub async fn get_power_characteristic(device: &Peripheral) -> Option<Characteristic> {
    if let Err(error) = device.discover_services().await {
        eprintln!("{error:?}");
        return None;
    };
    device
        .services()
        .into_iter()
        .filter(|service| service.uuid == LHV2_GATT_POWER_SERVICE)
        .next()
        .and_then(|service| {
            service
                .characteristics
                .into_iter()
                .filter(|char| char.uuid == LHV2_GATT_POWER_CHARACTERISTIC)
                .next()
        })
}
