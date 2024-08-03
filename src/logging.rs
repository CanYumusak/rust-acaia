use crate::constants::{Message, Settings};

impl Message {
    pub(crate) fn log(&self) {
        match self {
            Message::Weight { value } => println!("Weight: {}", value),
            Message::Heartbeat { value, time } => println!("Heartbeat - weight: {:?}, time: {:?}", value, time),
            Message::Timer { time } => println!("Timer: {}", time),
            Message::Button { button, value, time } => println!("Button: {} - weight: {:?}, time: {:?}", button, value, time),
            Message::Unknown { msg_type, payload } => println!("Unknown message type: {}, payload: {:?}", msg_type, payload),
        }
    }
}

impl Settings {
    pub(crate) fn log(&self) {
        println!(
            "settings: battery={} {} auto_off={} beep={}",
            self.battery,
            self.units.as_deref().unwrap_or("None"),
            self.auto_off,
            self.beep_on
        );
    }
}