use std::{collections::HashMap, sync::LazyLock};

use btleplug::platform::{Peripheral, PeripheralId};
use tokio::sync::Mutex;
use uuid::Uuid;

// 00001523-1212-efde-1523-785feabcd124
pub const LHV2_GATT_POWER_SERVICE: Uuid = Uuid::from_u128(0x000015231212efde1523785feabcd124);

// 00001525-1212-efde-1523-785feabcd124
pub const LHV2_GATT_POWER_CHARACTERISTIC: Uuid =
    Uuid::from_u128(0x000015251212efde1523785feabcd124);

pub static DEVICE_MAP: LazyLock<Mutex<HashMap<PeripheralId, Peripheral>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
