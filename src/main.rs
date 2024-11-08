use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use btleplug::api::Peripheral;
use futures::stream::StreamExt;
use tokio::time::{Instant, sleep};
use crate::acaia_scanner::AcaiaScanner;

mod constants;
mod logging;
mod decoding;
mod encoding;
mod acaia_scanner;
mod acaia_scale;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let scanner = AcaiaScanner::new().await.unwrap();
    let acaia = scanner.start_scan()
        .await
        .unwrap()
        .next()
        .await
        .unwrap();

    let acaia_arc = Arc::new(acaia);
    let handle = acaia_arc.clone().connect().await?;
    //
    // acaia_arc.tare().await;
    // acaia_arc.start_timer().await;
    // sleep(Duration::from_secs(5)).await;
    // acaia_arc.stop_timer().await;
    //
    // sleep(Duration::from_secs(2)).await;
    // acaia_arc.reset_timer().await;

    let start_time = Instant::now();
    let duration = Duration::from_secs(50);
    while start_time.elapsed() < duration {
        if acaia_arc.is_connected().await {
            let guard = acaia_arc.current_weight.lock().await;
            if let Some(weight) = *guard {
                println!("Weight: {:?}", weight);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        } else {
            break;
        }
    }
    println!("Finito");
    handle.await.unwrap();
    Ok(())
}
