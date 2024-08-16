use crate::constants::{MAGIC1, MAGIC2};

pub fn encode_event_data(payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(payload.len() + 1);
    bytes.push((payload.len() + 1) as u8);

    for &byte in payload {
        bytes.push(byte & 0xff);
    }

    encode(12, &bytes)
}

pub fn encode(msg_type: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0u8; 5 + payload.len()];
    bytes[0] = MAGIC1;
    bytes[1] = MAGIC2;
    bytes[2] = msg_type;

    let mut cksum1: u16 = 0;
    let mut cksum2: u16 = 0;

    for (i, &val) in payload.iter().enumerate() {
        bytes[3 + i] = val;
        if i % 2 == 0 {
            cksum1 += val as u16;
        } else {
            cksum2 += val as u16;
        }
    }

    bytes[payload.len() + 3] = (cksum1 & 0xFF) as u8;
    bytes[payload.len() + 4] = (cksum2 & 0xFF) as u8;

    bytes
}