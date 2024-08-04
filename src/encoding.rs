use btleplug::api::{Characteristic, Peripheral, WriteType};
use crate::constants::{MAGIC1, MAGIC2};
// def encodeGetSettings():
// """Settings are returned as a notification"""
// payload = [0]*16
// return encode(6,payload)
//
//
// pub async fn request_settings(device: &Peripheral, cmd_char: &Characteristic) {
//     let payload = &vec![0; 16];
//     let encoded = encode(6, payload).as_slice();
//     return device.write(cmd_char, encoded, WriteType::WithoutResponse)
//         .await
//         .unwrap();
// }
//
// pub async fn tare(device: &Peripheral, cmd_char: &Characteristic) {
//     let payload = &[0];
//     let encoded = encode(5, payload).as_slice();
//     return device.write(cmd_char, encoded, WriteType::WithoutResponse)
//         .await
//         .unwrap();
// }
//
// pub async fn start_timer(device: &Peripheral, cmd_char: &Characteristic) {
//     let payload = &[0, 0];
//     return send_action(device, cmd_char, payload);
// }
//
// pub async fn stop_timer(device: &Peripheral, cmd_char: &Characteristic) {
//     let payload = &[0, 2];
//     return send_action(device, cmd_char, payload);
// }
//
// pub async fn reset_timer(device: &Peripheral, cmd_char: &Characteristic) {
//     let payload = &[0, 1];
//     return send_action(device, cmd_char, payload);
// }
//
// async fn send_action(device: &Peripheral, cmd_char: &Characteristic, payload: &[u8]) {
//     return device.write(cmd_char, &*encode(13, payload), WriteType::WithoutResponse)
//         .await
//         .unwrap();
// }
//
// pub async fn ident(device: &Peripheral, cmd_char: &Characteristic) {
//     let payload = &[0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d, 0x2d];
//     let encoded = encode(11, payload).as_slice();
//     device.write(cmd_char, encoded, WriteType::WithoutResponse)
//         .await
//         .unwrap();
//     println!("Sent ident");
// }
//
// pub async fn request_heartbeat(device: &Peripheral, cmd_char: &Characteristic) {
//     let payload = &[0, 1, 1, 2, 2, 5, 3, 4];
//
//     let vec = encode_event_data(payload);
//     device.write(cmd_char, &*vec, WriteType::WithoutResponse)
//         .await
//         .unwrap();
//     println!("Sent Notificaton Request");
// }
//
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