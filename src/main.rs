extern crate uinput;

use std::net::UdpSocket;
use uinput::event::keyboard;
use std::thread;
use std::time::Duration;
use uinput::event::controller::Controller::{Mouse};
use uinput::event::controller::Mouse::{Left, Right, Middle};
use uinput::event::Event::{Controller, Relative};
use uinput::event::relative::Position::{X, Y};
use uinput::event::relative::Relative::{Position};
use uinput::event::relative::Wheel;

fn to_num(one_byte: u8) -> i32 {
    let mut num = one_byte as i32;
    if num > 128 {
        num -= 256;
    }
    return num;
}


const LEFT_MOUSE: u8 = 90;
const RIGHT_MOUSE: u8 = 91;
const MIDDLE_MOUSE: u8 = 92;

const LEFT_MOUSE_CODE: i32 = 272;
const RIGHT_MOUSE_CODE: i32 = 273;
const MIDDLE_MOUSE_CODE: i32 = 274;


fn to_button(one_byte: u8) -> i32 {
    match one_byte {
        LEFT_MOUSE => LEFT_MOUSE_CODE,
        RIGHT_MOUSE => RIGHT_MOUSE_CODE,
        MIDDLE_MOUSE => MIDDLE_MOUSE_CODE,
        _ => one_byte as i32,
    }
}

fn main() {
    let mut device = uinput::default().unwrap()
        .name("test").unwrap()
        // It's necessary to enable any mouse button. Otherwise Relative events would not work.
        .event(Controller(Mouse(Left))).unwrap()
        .event(Controller(Mouse(Right))).unwrap()
        .event(Controller(Mouse(Middle))).unwrap()
        .event(Relative(Position(X))).unwrap()
        .event(Relative(Position(Y))).unwrap()
        .event(uinput::event::Keyboard::All).unwrap()
        .event(Wheel::Horizontal).unwrap()
        .event(Wheel::Vertical).unwrap()
        .create().unwrap();

    let socket = match UdpSocket::bind("0.0.0.0:5005") {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e)
    };

    let mut msg = [0; 2];
    loop {
        let (msg_len, _) = socket.recv_from(&mut msg).unwrap();
        if msg_len == 2 {
            if msg[0] == 128 {
                let y = to_num(msg[1]);
                device.send(Wheel::Vertical, -y).unwrap();
            } else if msg[1] == 128 {
                let x = to_num(msg[1]);
                device.send(Wheel::Horizontal, x).unwrap();
            } else {
                let x = to_num(msg[0]);
                let y = to_num(msg[1]);
                device.send(X, x).unwrap();
                device.send(Y, -y).unwrap();
            }
        } else if msg_len == 1 {
            if msg[0] > 128 {
                msg[0] -= 128;
                device.press(to_button(msg[0])).unwrap();
            } else {
                device.release(to_button(msg[0])).unwrap();
            }
        }
        device.synchronize().unwrap();
    }
}