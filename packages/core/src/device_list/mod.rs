use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use btleplug::{
    api::{Central, CentralEvent, Peripheral as _, ScanFilter},
    platform::{Adapter, PeripheralId},
};
use futures::StreamExt;
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time::sleep,
};

use crate::{device::Device, get_default_adapter, DeviceInfo, DeviceLocalStatus};

/// Can be cloned and will retain references to the same devices
#[derive(Clone, Debug)]
pub struct DeviceList {
    adapter: Adapter,
    map: Arc<Mutex<HashMap<PeripheralId, Device>>>,
}

impl DeviceList {
    pub async fn init() -> crate::Result<Self> {
        Ok(Self {
            map: Arc::new(Mutex::new(HashMap::new())),
            adapter: get_default_adapter().await?,
        })
    }

    pub fn get_adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn get_device_map(&self) -> Arc<Mutex<HashMap<PeripheralId, Device>>> {
        self.map.clone()
    }

    pub fn get_device(&self, id: &PeripheralId) -> Option<Device> {
        self.map
            .clone()
            .lock()
            .expect("Device map mutex must not be poisoned")
            .get(id)
            .cloned()
    }

    pub fn start_scan(&self, duration: u64) -> crate::Result<Receiver<DeviceInfo>> {
        let (tx, rx) = channel(1);

        let refresh = self
            .get_device_map()
            .lock()
            .expect("Device map mutex must not be poisoned")
            .clone()
            .into_values();
        for device in refresh {
            let tx_clone = tx.clone();
            tokio::spawn(async move { device.fetch_remote_status(tx_clone).await });
        }

        let devices = self.clone();
        tokio::spawn(async move {
            let adapter = devices.get_adapter();

            let timer = Box::pin(sleep(Duration::from_secs(duration)));
            let mut stream = adapter.events().await?.take_until(timer);

            adapter.start_scan(ScanFilter::default()).await?;
            while let Some(evt) = stream.next().await {
                if let CentralEvent::DeviceDiscovered(id) = evt {
                    let future = handle_discovered_device(devices.clone(), tx.clone(), id);
                    tokio::spawn(future);
                }
            }
            adapter.stop_scan().await?;

            Ok::<(), crate::Error>(())
        });

        Ok(rx)
    }
}

async fn handle_discovered_device(
    list: DeviceList,
    tx: Sender<DeviceInfo>,
    id: PeripheralId,
) -> crate::Result<()> {
    if list
        .get_device_map()
        .lock()
        .expect("Device map mutex must not be poisoned")
        .contains_key(&id)
    {
        return Ok(());
    }
    let peripheral = list.get_adapter().peripheral(&id).await?;
    let maybe_name = peripheral
        .properties()
        .await
        .unwrap_or(None)
        .and_then(|props| props.local_name);
    let valid_name = maybe_name
        .clone()
        .and_then(|name| match name.starts_with("LHB-") {
            true => Some(name),
            false => None,
        });
    let Some(name) = valid_name else {
        let addr = peripheral.address().to_string();
        let _ = tx
            .send(DeviceInfo {
                id,
                name: maybe_name.unwrap_or(format!("[{addr}]")),
                addr,
                local: Some(DeviceLocalStatus::Ignored),
                remote: None,
            })
            .await;
        return Ok(());
    };

    let device = Device::new(peripheral.clone(), name.clone());
    list.map
        .lock()
        .expect("Device map mutex must not be poisoned")
        .insert(id, device.clone());

    device.fetch_remote_status(tx).await?;

    Ok(())
}
