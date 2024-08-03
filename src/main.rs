mod constants;
mod logging;
mod decoding;
mod encoding;

use constants::{Message, Settings};

use btleplug::api::{bleuuid::uuid_from_u16, Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::error::Error;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;
use futures::stream::StreamExt;


const CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0x2A80);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await.unwrap();

    // get the first bluetooth adapter
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    // start scanning for devices
    let filter = ScanFilter::default();
    central.start_scan(filter).await?;
    // instead of waiting, you can use central.events() to get a stream which will
    // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    time::sleep(Duration::from_secs(2)).await;

    // find the device we're interested in
    let acaia = find_acaia_device(&central).await.unwrap();
    acaia.connect().await?;
    acaia.discover_services().await?;

    let chars = acaia.characteristics();
    let cmd_char = chars.iter().find(|c| c.uuid == CHARACTERISTIC_UUID).unwrap();


    acaia.subscribe(&cmd_char).await?;
    let acaia_clone = acaia.clone();
    let handle = tokio::spawn(async move {
        if let Err(e) = handle_notifications(&acaia_clone).await {
            eprintln!("Error in notification handler: {:?}", e);
        }
    });

    encoding::ident(&acaia, &cmd_char).await;
    encoding::request_heartbeat(&acaia, &cmd_char).await;

    handle.await.unwrap();
    Ok(())
}

async fn handle_notifications(mut acaia: &Peripheral) -> Result<(), Box<dyn Error>> {
    println!("Handle Notifications");

    let mut notification_stream = acaia.notifications().await?;

    while let Some(notification) = notification_stream.next().await {
        if notification.uuid == CHARACTERISTIC_UUID {
            let vec = &notification.value;
            println!("Received data: {:?}", vec);
            if vec.len() > 0 && vec[0] == 9 {
                let settings = Settings::try_from(vec.as_slice());
                if let Ok(settings) = settings {
                    settings.log();
                }
            } else if vec.len() > 0 && vec[0] == 8 {
                let message = Message::try_from(vec.as_slice());
                if let Ok(message) = message {
                    message.log();
                }
            }
            // Process the notification data here
        }
    }
    println!("End");

    Ok(())
}


async fn find_acaia_device(central: &Adapter) -> Option<Peripheral> {
    let devices_start_names = ["ACAIA"];

    let peripherals = central.peripherals().await.unwrap_or_default();
    for p in peripherals {
        if let Ok(Some(properties)) = p.properties().await {
            if let Some(local_name) = properties.local_name {
                if devices_start_names.iter().any(|&prefix| local_name.starts_with(prefix)) {
                    return Some(p);
                }
            }
        }
    }
    None
}

