extern crate mouse_keyboard_input;

use std::net::UdpSocket;
use std::thread;
use std::thread::JoinHandle;
use mouse_keyboard_input::VirtualDevice;
use mouse_keyboard_input::key_codes::*;


type SharedDevice = VirtualDevice;
type Coord = i32;
type Button = u16;

fn to_num(one_byte: u8) -> Coord {
    let mut num = one_byte as Coord;
    if num > 128 {
        num -= 256;
    }
    num
}

const LEFT_MOUSE: u8 = 90;
const RIGHT_MOUSE: u8 = 91;
const MIDDLE_MOUSE: u8 = 92;

fn to_button(one_byte: u8) -> Button {
    match one_byte {
        LEFT_MOUSE => BTN_LEFT,
        RIGHT_MOUSE => BTN_RIGHT,
        MIDDLE_MOUSE => BTN_MIDDLE,
        _ => one_byte as Button,
    }
}

fn parse_button(socket: UdpSocket, device: &mut SharedDevice) {
    let mut msg = [0; 1];
    let mut button: Button;

    loop {
        socket.recv_from(&mut msg).unwrap();

        if msg[0] > 128 {
            msg[0] -= 128;
            button = to_button(msg[0]);
            device.press(button).unwrap();
        } else {
            button = to_button(msg[0]);
            device.release(button).unwrap();
        }
    }
}

fn parse_scroll(socket: UdpSocket, device: &mut SharedDevice) {
    let mut msg = [0; 1];
    let mut y: Coord;

    loop {
        socket.recv_from(&mut msg).unwrap();

        y = to_num(msg[0]);

        device.scroll_vertical(y).unwrap();
    }
}

fn parse_mouse(socket: UdpSocket, device: &mut SharedDevice) {
    let mut msg = [0; 2];

    let mut x: Coord;
    let mut y: Coord;

    loop {
        socket.recv_from(&mut msg).unwrap();

        x = to_num(msg[0]);
        y = to_num(msg[1]);

        device.move_mouse(x, y).unwrap();
    }
}

fn create_udp_thread(parse_func: fn(UdpSocket, &mut SharedDevice), port: u16) -> JoinHandle<()> {
    thread::spawn(move || {
        let address = "0.0.0.0";

        let socket = match UdpSocket::bind((address, port)) {
            Ok(s) => s,
            Err(e) => panic!("couldn't bind socket: {}", e)
        };

        println!("UDP at port {}:", port);

        let mut device = VirtualDevice::new();

        parse_func(socket, &mut device);
    })
}

fn main() {
    create_udp_thread(parse_button, 5009);
    create_udp_thread(parse_scroll, 5007);
    create_udp_thread(parse_mouse, 5005).join().unwrap();
}