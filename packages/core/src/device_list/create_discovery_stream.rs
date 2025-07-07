use std::time::Duration;

use async_trait::async_trait;
use btleplug::api::{Central, CentralEvent, Peripheral as _, ScanFilter};
use btleplug::platform::{Peripheral, PeripheralId};
use futures::{Stream, StreamExt};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::sleep;

use crate::dto::{DevicePowerStatus, DeviceUpdatePayload};
use crate::util::assert_power_characteristic;

use super::DeviceList;

#[async_trait]
pub trait CreateDiscoveryStream {
    async fn create_discovery_stream(
        &self,
        duration: u64,
    ) -> crate::Result<Receiver<DeviceUpdatePayload>>;
}

#[async_trait]
impl CreateDiscoveryStream for DeviceList {
    async fn create_discovery_stream(
        &self,
        duration: u64,
    ) -> crate::Result<Receiver<DeviceUpdatePayload>> {
        let (tx, rx) = channel(1);
        let adapter = self.assert_adapter()?;
        let stream = adapter.events().await?;
        adapter.start_scan(ScanFilter::default()).await?;
        self.map.lock().await.clear();
        let list = self.clone();
        tokio::spawn(handle_discovery_stream(list, tx, stream, duration));
        Ok(rx)
    }
}

async fn handle_discovery_stream<S: Stream<Item = CentralEvent> + Send + Unpin>(
    list: DeviceList,
    tx: Sender<DeviceUpdatePayload>,
    stream: S,
    duration: u64,
) -> crate::Result<()> {
    let timer = Box::pin(sleep(Duration::from_secs(duration)));
    let mut limited = stream.take_until(timer);
    while let Some(evt) = limited.next().await {
        if let CentralEvent::DeviceDiscovered(id) = evt {
            let future = handle_dicovered_device(list.clone(), tx.clone(), id);
            tokio::spawn(future);
        };
    }
    let _ = list.assert_adapter()?.stop_scan().await;
    Ok(())
}

async fn handle_dicovered_device(
    list: DeviceList,
    tx: Sender<DeviceUpdatePayload>,
    id: PeripheralId,
) -> crate::Result<()> {
    if list.map.lock().await.contains_key(&id) {
        return Ok(());
    }
    let device = list.assert_adapter()?.peripheral(&id).await?;
    let Some(name) = device
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

    list.map.lock().await.insert(id.clone(), device.clone());
    tx.send(DeviceUpdatePayload {
        id: id.clone(),
        addr: device.address().to_string(),
        name: Some(name.clone()),
        power: DevicePowerStatus::Loading,
    })
    .await?;
    if !device.is_connected().await? {
        device.connect().await?;
    }
    let status = get_device_status(&device).await?;
    device.disconnect().await?;
    tx.send(DeviceUpdatePayload {
        id,
        addr: device.address().to_string(),
        name: Some(name),
        power: status,
    })
    .await?;

    Ok(())
}

async fn get_device_status(device: &Peripheral) -> crate::Result<DevicePowerStatus> {
    let char = assert_power_characteristic(device).await?;
    let bytes = device.read(&char).await?;
    Ok(DevicePowerStatus::from(bytes))
}
