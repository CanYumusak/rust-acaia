use std::error::Error;
use std::sync::Arc;
use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{Characteristic, Peripheral, WriteType};
use futures::StreamExt;
use btleplug::platform::Peripheral as PeripheralStruct;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use uuid::Uuid;
use tokio::time::{sleep, Duration, timeout};
use crate::constants::{Message, Settings};
use crate::encoding::{encode, encode_event_data};

const CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0x2A80);

pub struct AcaiaScale {
    peripheral: PeripheralStruct,
    pub(crate) current_weight: Mutex<Option<f32>>,
    pub(crate) current_time: Mutex<Option<f32>>,
}

impl AcaiaScale {
    pub fn new(peripheral: PeripheralStruct) -> Self {
        let current_weight = Mutex::new(None);
        let current_time = Mutex::new(None);
        Self {
            peripheral,
            current_weight,
            current_time,
        }
    }

    pub(crate) async fn connect(self: Arc<Self>) -> btleplug::Result<JoinHandle<()>> {
        self.peripheral.connect().await?;
        self.peripheral.discover_services().await?;

        let cmd_char = self.get_command_characteristic();
        self.peripheral.subscribe(&cmd_char).await?;


        let handle = tokio::spawn({
            let me = Arc::clone(&self);
            async move {
                if let Err(e) = me.handle_notifications().await {
                    eprintln!("Error in notification handler: {:?}", e);
                }
            }
        });

        self.ident().await;
        self.request_heartbeat().await;
        sleep(Duration::from_millis(100)).await;

        Ok(handle)
    }

    async fn handle_notifications(&self) -> Result<(), Box<dyn Error>> {
        let mut notification_stream = self.peripheral.notifications().await?;
        while let Some(notification) = notification_stream.next().await {
            if notification.uuid == CHARACTERISTIC_UUID {
                let vec = &notification.value;
                if vec.len() > 0 && vec[0] == 9 {
                    let settings = Settings::try_from(vec.as_slice());
                    if let Ok(settings) = settings {
                        settings.log();
                    }
                } else if vec.len() > 0 && vec[0] == 8 {
                    if let Ok(message) = Message::try_from(vec.as_slice()) {
                        match message {
                            Message::Weight { value: w } => { self.set_weight(w).await }
                            Message::Heartbeat { value: w, time: t } => {
                                if let Some(w) = w { self.set_weight(w).await; }
                                if let Some(t) = t { self.set_time(t).await; }
                            }
                            Message::Timer { time: t } => { self.set_time(t).await; }
                            _ => {}  // Handle other message types as needed
                        }
                    }
                }
            }
        }
        println!("End");

        Ok(())
    }

    pub(crate) async fn is_connected(&self) -> bool {
        match timeout(Duration::from_millis(200), self.peripheral.is_connected()).await {
            Ok(Ok(connected)) => connected,
            Ok(Err(_)) => false, // Connection check failed
            Err(_) => false // Timeout occurred
        }
    }

    async fn set_weight(&self, weight: f32) {
        *self.current_weight.lock().await = Some(weight);
    }

    async fn set_time(&self, time: f32) {
        *self.current_time.lock().await = Some(time);
    }

    pub async fn request_settings(&self) {
        let cmd_char = self.get_command_characteristic();
        let payload = &vec![0; 16];
        self.peripheral.write(&cmd_char, encode(6, payload).as_slice(), WriteType::WithoutResponse)
            .await
            .unwrap();

        println!("Sent Tare")
    }

    pub async fn tare(&self) {
        let cmd_char = self.get_command_characteristic();
        self.peripheral.write(&cmd_char, &*encode(4, &[0]), WriteType::WithoutResponse)
            .await
            .unwrap();

        println!("Sent Tare")
    }

    pub async fn start_timer(&self) {
        let payload = &[0, 0];
        self.send_action(payload).await;
        println!("Sent Start Timer")
    }

    pub async fn stop_timer(&self) {
        let payload = &[0, 2];
        self.send_action(payload).await;

        println!("Sent Stop Timer")
    }

    pub async fn reset_timer(&self) {
        let payload = &[0, 1];
        self.send_action(payload).await;

        println!("Sent Reset Timer")
    }

    pub async fn ident(&self) {
        let cmd_char = self.get_command_characteristic();
        let payload = &[0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d];
        self.peripheral.write(&cmd_char, encode(11, payload).as_slice(), WriteType::WithoutResponse)
            .await
            .unwrap();
        println!("Sent ident");
    }

    pub async fn request_heartbeat(&self) {
        let cmd_char = self.get_command_characteristic();
        let payload = &[0, 1, 1, 2, 2, 5, 3, 4];

        let vec = encode_event_data(payload);
        self.peripheral.write(&cmd_char, &*vec, WriteType::WithoutResponse)
            .await
            .unwrap();
        println!("Sent Notificaton Request");
    }

    async fn send_action(&self, payload: &[u8]) {
        let cmd_char = self.get_command_characteristic();
        return self.peripheral.write(&cmd_char, &*encode(13, payload), WriteType::WithoutResponse)
            .await
            .unwrap();
    }

    fn get_command_characteristic(&self) -> Characteristic {
        let chars = self.peripheral.characteristics();
        return chars.into_iter().find(|c| c.uuid == CHARACTERISTIC_UUID).unwrap();
    }
}