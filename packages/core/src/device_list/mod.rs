use std::{collections::HashMap, sync::Arc, time::Duration};

use btleplug::{
    api::{Central, CentralEvent, Peripheral as _, ScanFilter},
    platform::{Adapter, PeripheralId},
};
use futures::StreamExt;
use tokio::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
    time::sleep,
};

use crate::{device::Device, get_default_adapter, DevicePowerStatus, DeviceUpdatePayload};

/// All clones share the same reference pointers
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

    pub fn get_adapter<'a>(&'a self) -> &'a Adapter {
        &self.adapter
    }

    pub fn get_device_map(&self) -> Arc<Mutex<HashMap<PeripheralId, Device>>> {
        self.map.clone()
    }

    pub async fn get_device(&self, id: &PeripheralId) -> Option<Device> {
        self.map.clone().lock().await.get(&id).cloned()
    }

    pub async fn start_scan(&self, duration: u64) -> crate::Result<Receiver<DeviceUpdatePayload>> {
        let (tx, rx) = channel(1);
        let devices = self.clone();

        tokio::spawn(async move {
            let adapter = devices.get_adapter();

            let timer = Box::pin(sleep(Duration::from_secs(duration)));
            let mut stream = adapter.events().await?.take_until(timer);

            adapter.start_scan(ScanFilter::default()).await?;
            while let Some(evt) = stream.next().await {
                if let CentralEvent::DeviceDiscovered(id) = evt {
                    let future = handle_dicovered_device(devices.clone(), tx.clone(), id);
                    tokio::spawn(future);
                };
            }
            adapter.stop_scan().await?;

            Ok::<(), crate::Error>(())
        });

        Ok(rx)
    }
}

async fn handle_dicovered_device(
    list: DeviceList,
    tx: Sender<DeviceUpdatePayload>,
    id: PeripheralId,
) -> crate::Result<()> {
    if list.map.lock().await.contains_key(&id) {
        return Ok(());
    }
    let peripheral = list.get_adapter().peripheral(&id).await?;
    let Some(name) = peripheral
        .properties()
        .await?
        .and_then(|props| props.local_name)
        .and_then(|name| match name.starts_with("LHB-") {
            true => Some(name),
            false => None,
        })
    else {
        return Ok(());
    };

    let device = Device::new(peripheral.clone(), name.clone());
    list.map.lock().await.insert(id, device.clone());
    tx.send(DeviceUpdatePayload::from_device(
        &device,
        DevicePowerStatus::Loading,
    ))
    .await?;
    device.ensure_connected().await?;
    let power = device.get_device_status().await?;
    tx.send(DeviceUpdatePayload::from_device(&device, power))
        .await?;

    Ok(())
}
