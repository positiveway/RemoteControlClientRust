extern crate mouse_keyboard_input;

use std::net::UdpSocket;
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, Instant};
use mouse_keyboard_input::*;

type Byte = u8;

fn to_num(one_byte: Byte) -> Coord {
    let mut num = one_byte as Coord;
    if num > 128 {
        num -= 256;
    }
    num
}

const LEFT_MOUSE: Button = 90;
const RIGHT_MOUSE: Button = 91;
const MIDDLE_MOUSE: Button = 92;

fn to_button(one_byte: Button) -> Button {
    match one_byte {
        LEFT_MOUSE => BTN_LEFT,
        RIGHT_MOUSE => BTN_RIGHT,
        MIDDLE_MOUSE => BTN_MIDDLE,
        _ => one_byte as Button,
    }
}

fn parse_btn_press(socket: UdpSocket, sender: &ChannelSender) {
    let mut msg = [0; 1];

    loop {
        socket.recv_from(&mut msg).unwrap();
        send_press(msg[0] as Button, sender).unwrap();
    }
}

fn parse_btn_release(socket: UdpSocket, sender: &ChannelSender) {
    let mut msg = [0; 1];

    loop {
        socket.recv_from(&mut msg).unwrap();
        send_release(msg[0] as Button, sender).unwrap();
    }
}

fn parse_scroll(socket: UdpSocket, sender: &ChannelSender) {
    let mut msg = [0; 1];
    let mut move_by: Coord;

    loop {
        socket.recv_from(&mut msg).unwrap();

        move_by = to_num(msg[0]);

        send_scroll_y(move_by, sender).unwrap();
    }
}

fn parse_mouse_x(socket: UdpSocket, sender: &ChannelSender) {
    let mut msg = [0; 1];
    let mut move_by: Coord;

    loop {
        socket.recv_from(&mut msg).unwrap();

        move_by = to_num(msg[0]);

        send_mouse_move_x(move_by, sender).unwrap();
    }
}

fn parse_mouse_y(socket: UdpSocket, sender: &ChannelSender) {
    let mut msg = [0; 1];
    let mut move_by: Coord;

    loop {
        socket.recv_from(&mut msg).unwrap();

        move_by = to_num(msg[0]);

        send_mouse_move_y(move_by, sender).unwrap();
    }
}


fn create_udp_thread(parse_func: fn(UdpSocket, &ChannelSender), port: u16, sender: ChannelSender) -> JoinHandle<()> {
    thread::spawn(move || {
        let address = "0.0.0.0";

        let socket = match UdpSocket::bind((address, port)) {
            Ok(s) => s,
            Err(e) => panic!("couldn't bind socket: {}", e)
        };

        println!("UDP at port {}:", port);

        parse_func(socket, &sender);
    })
}

const WRITING_INTERVAL: Duration = Duration::from_millis(1);


const MOUSE_PORT_X: u16 = 5004;
const MOUSE_PORT_Y: u16 = 5005;

const SCROLL_PORT_X: u16 = 5006;
const SCROLL_PORT_Y: u16 = 5007;

const PRESS_BTN_PORT: u16 = 5008;
const RELEASE_BTN_PORT: u16 = 5009;

fn main() {
    let mut device = VirtualDevice::default().unwrap();

    create_udp_thread(parse_btn_press, PRESS_BTN_PORT, device.sender.clone());
    create_udp_thread(parse_btn_release, RELEASE_BTN_PORT, device.sender.clone());
    create_udp_thread(parse_mouse_x, MOUSE_PORT_X, device.sender.clone());
    create_udp_thread(parse_mouse_y, MOUSE_PORT_Y, device.sender.clone());
    create_udp_thread(parse_scroll, SCROLL_PORT_Y, device.sender.clone());

    device.write_from_channel_every_ms();
}