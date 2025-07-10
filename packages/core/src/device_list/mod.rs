use std::{collections::HashMap, sync::Arc, time::Duration};

use btleplug::{
    api::{Central, CentralEvent, Peripheral as _, ScanFilter},
    platform::{Adapter, PeripheralId},
};
use futures::{StreamExt, TryFutureExt};
use tokio::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
    time::sleep,
};

use crate::{device::Device, get_default_adapter, DeviceInfo, DeviceLocalStatus, SendDeviceStatus};

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

    pub fn get_adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn get_device_map(&self) -> Arc<Mutex<HashMap<PeripheralId, Device>>> {
        self.map.clone()
    }

    pub async fn get_device(&self, id: &PeripheralId) -> Option<Device> {
        self.map.clone().lock().await.get(id).cloned()
    }

    pub fn start_scan(&self, duration: u64) -> crate::Result<Receiver<DeviceInfo>> {
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
                }
            }
            adapter.stop_scan().await?;

            Ok::<(), crate::Error>(())
        });

        Ok(rx)
    }
}

async fn handle_dicovered_device(
    list: DeviceList,
    tx: Sender<DeviceInfo>,
    id: PeripheralId,
) -> crate::Result<()> {
    if list.map.lock().await.contains_key(&id) {
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
    list.map.lock().await.insert(id, device.clone());

    let _ = tx
        .send_device_local_status(&device, DeviceLocalStatus::Initializing)
        .await;
    device
        .ensure_connected()
        .and_then(async |()| {
            let _ = tx
                .send_device_local_status(&device, DeviceLocalStatus::Connected)
                .await;
            Ok(())
        })
        .or_else(async |err| {
            let _ = tx
                .send_device_local_status(&device, DeviceLocalStatus::FailConnection)
                .await;
            Err(err)
        })
        .await?;
    let remote = device
        .get_device_remote_status()
        .or_else(async |err| {
            let _ = tx
                .send_device_local_status(&device, DeviceLocalStatus::FailVerify)
                .await;
            Err(err)
        })
        .await?;
    tx.send_device_remote_status(&device, remote).await?;
    let disconnected = device.disconnect().await;
    let status = match disconnected {
        Ok(()) => DeviceLocalStatus::Disconnected,
        Err(_) => DeviceLocalStatus::FailConnection,
    };
    let _ = tx.send_device_local_status(&device, status).await;
    Ok(())
}
