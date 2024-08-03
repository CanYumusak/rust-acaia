
pub(crate) const MAGIC1: u8 = 0xef;
pub(crate) const MAGIC2: u8 = 0xdd;

// #[derive(Debug)]
// enum Message {
//     Settings(Settings),
//     // Other message types can be added here
//     Unknown(Vec<u8>),
// }

enum MsgType {
    Weight,
    Heartbeat,
    Time,
    Button,
}

impl MsgType {
    fn value(&self) -> u8 {
        match *self {
            MsgType::Weight => 5,
            MsgType::Heartbeat => 11,
            MsgType::Time => 7,
            MsgType::Button => 8,
        }
    }
}

enum ButtonEvent {
    Tare,
    Start,
    Stop,
    Reset,
}

impl ButtonEvent {
    fn value(&self) -> (u8, u8) {
        match *self {
            ButtonEvent::Tare => (0, 5),
            ButtonEvent::Start => (8, 5),
            ButtonEvent::Stop => (10, 7),
            ButtonEvent::Reset => (9, 7),
        }
    }
}

enum Units {
    Grams,
    Ounces,
}

#[derive(Debug)]
pub(crate) struct Settings {
    pub(crate) battery: u8,
    pub(crate) units: Option<String>,
    pub(crate) auto_off: u8,
    pub(crate) beep_on: bool,
}


#[derive(Debug)]
pub(crate) enum Message {
    Weight { value: f32 },
    Heartbeat { value: Option<f32>, time: Option<f32> },
    Timer { time: f32 },
    Button { button: String, value: Option<f32>, time: Option<f32> },
    Unknown { msg_type: u8, payload: Vec<u8> },
}


