use std::error::Error;
use std::pin::Pin;
use std::sync::Arc;
use async_stream::stream;
use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use futures::stream::{Stream, StreamExt};

use crate::acaia_scale::AcaiaScale;

pub struct AcaiaScanner {
    manager: Manager,
}

impl AcaiaScanner {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let manager = Manager::new().await?;
        Ok(Self { manager })
    }

    pub async fn start_scan(
        &self
    ) -> btleplug::Result<Pin<Box<dyn Stream<Item=AcaiaScale> + Send>>> {
        let adapters = self.manager.adapters().await?;
        let central = Arc::new(adapters.into_iter().next().unwrap());

        central.start_scan(ScanFilter::default()).await?;

        self.create_scale_stream(central).await
    }

    async fn create_scale_stream(
        &self,
        central: Arc<Adapter>
    ) -> btleplug::Result<Pin<Box<dyn Stream<Item=AcaiaScale> + Send>>> {
        let events = central.events().await?;
        let stream = events.filter_map(move |event| {
            let central = Arc::clone(&central);

            async move {
                if let CentralEvent::DeviceDiscovered(id) = event {
                    let peripheral = central.peripheral(&id).await.ok()?;
                    if Self::is_acaia_scale(&peripheral).await {
                        Some(AcaiaScale::new(peripheral))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        });

        Ok(Box::pin(stream))
    }

    async fn is_acaia_scale(peripheral: &impl Peripheral) -> bool {
        peripheral.properties().await
            .ok()
            .flatten()
            .and_then(|props| props.local_name)
            .map_or(false, |name| name.starts_with("ACAIA"))
    }
}
