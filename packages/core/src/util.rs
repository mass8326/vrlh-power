use btleplug::api::{Characteristic, Peripheral as _};
use btleplug::platform::Peripheral;

use crate::constants::{LHV2_GATT_POWER_CHARACTERISTIC, LHV2_GATT_POWER_SERVICE};

pub async fn assert_power_characteristic(device: &Peripheral) -> crate::Result<Characteristic> {
    device.discover_services().await?;
    device
        .services()
        .into_iter()
        .filter(|service| service.uuid == LHV2_GATT_POWER_SERVICE)
        .next()
        .ok_or(crate::Error::Vrlh("Could not verify power service!"))
        .and_then(|service| {
            service
                .characteristics
                .into_iter()
                .filter(|char| char.uuid == LHV2_GATT_POWER_CHARACTERISTIC)
                .next()
                .ok_or(crate::Error::Vrlh("Could not verify power charateristic!"))
        })
}
