use crate::constants::{Message, Settings};

impl TryFrom<&[u8]> for Settings {
    type Error = &'static str;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if payload.len() != 11 {
            return Err("Payload must be 11 bytes long");
        }

        let units = match payload[2] {
            2 => Some("grams".to_string()),
            5 => Some("ounces".to_string()),
            _ => None,
        };

        Ok(Settings {
            battery: payload[1] & 0x7F,
            units,
            auto_off: payload[4] * 5,
            beep_on: payload[6] == 1,
        })
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = &'static str;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if payload.is_empty() {
            return Err("Payload is empty");
        }

        let msg_type = payload[1];
        let data = &payload[2..];

        match msg_type {
            5 => Ok(Message::Weight { value: decode_weight(data)? }),
            11 => {
                if data.len() < 3 {
                    return Err("Payload too short for heartbeat");
                }
                match data[2] {
                    5 => Ok(Message::Heartbeat {
                        value: Some(decode_weight(&data[3..])?),
                        time: None,
                    }),
                    7 => Ok(Message::Heartbeat {
                        value: None,
                        time: Some(decode_time(&data[3..])?),
                    }),
                    _ => Err("Unknown heartbeat type"),
                }
            },
            7 => Ok(Message::Timer { time: decode_time(data)? }),
            8 => {
                if data.len() < 2 {
                    return Err("Payload too short for button message");
                }
                match (data[0], data[1]) {
                    (0, 5) => Ok(Message::Button {
                        button: "tare".to_string(),
                        value: Some(decode_weight(&data[2..])?),
                        time: None,
                    }),
                    (8, 5) => Ok(Message::Button {
                        button: "start".to_string(),
                        value: Some(decode_weight(&data[2..])?),
                        time: None,
                    }),
                    (10, 7) => Ok(Message::Button {
                        button: "stop".to_string(),
                        time: Some(decode_time(&data[2..6])?),
                        value: Some(decode_weight(&data[6..])?),
                    }),
                    (9, 7) => Ok(Message::Button {
                        button: "reset".to_string(),
                        time: Some(decode_time(&data[2..6])?),
                        value: Some(decode_weight(&data[6..])?),
                    }),
                    _ => Ok(Message::Button {
                        button: "unknownbutton".to_string(),
                        value: None,
                        time: None,
                    }),
                }
            },
            _ => Ok(Message::Unknown {
                msg_type,
                payload: data.to_vec(),
            }),
        }
    }
}

fn decode_weight(weight_payload: &[u8]) -> Result<f32, &'static str> {
    if weight_payload.len() < 6 {
        return Err("Weight payload too short");
    }
    let mut value = ((weight_payload[1] as u16) << 8 | weight_payload[0] as u16) as f32;
    let unit = weight_payload[4];
    value /= match unit {
        1 => 10.0,
        2 => 100.0,
        3 => 1000.0,
        4 => 10000.0,
        _ => return Err("Invalid unit value"),
    };
    if (weight_payload[5] & 0x02) == 0x02 {
        value *= -1.0;
    }
    Ok(value)
}

fn decode_time(time_payload: &[u8]) -> Result<f32, &'static str> {
    if time_payload.len() < 3 {
        return Err("Time payload too short");
    }
    let value = (time_payload[0] as f32) * 60.0
        + time_payload[1] as f32
        + (time_payload[2] as f32) / 10.0;
    Ok(value)
}