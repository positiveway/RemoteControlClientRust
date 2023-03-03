extern crate mouse_keyboard_input;

use std::borrow::{Borrow, BorrowMut};
use std::net::UdpSocket;
use std::ops::{Sub};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, SystemTime};
use mouse_keyboard_input::{VirtualDevice, Button, Coord, ChannelSender, send_press, send_release, send_scroll_vertical, send_mouse_move};
use mouse_keyboard_input::key_codes::*;

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

fn parse_button(socket: UdpSocket, sender: ChannelSender) {
    let mut msg = [0; 1];
    let mut button: Button;

    loop {
        socket.recv_from(&mut msg).unwrap();
        button = msg[0] as Button;

        if button > 128 {
            button -= 128;
            button = to_button(button);
            send_press(button, sender.to_owned()).unwrap();
        } else {
            button = to_button(button);
            send_release(button, sender.to_owned()).unwrap();
        }
    }
}

fn parse_scroll(socket: UdpSocket, sender: ChannelSender) {
    let mut msg = [0; 1];
    let mut y: Coord;

    loop {
        socket.recv_from(&mut msg).unwrap();


        y = to_num(msg[0]);

        send_scroll_vertical(y, sender.to_owned()).unwrap();
    }
}

fn parse_mouse(socket: UdpSocket, sender: ChannelSender) {
    let mut msg = [0; 2];

    let mut x: Coord;
    let mut y: Coord;

    loop {
        socket.recv_from(&mut msg).unwrap();

        x = to_num(msg[0]);
        y = to_num(msg[1]);

        send_mouse_move(x,y, sender.to_owned()).unwrap();
    }
}

fn create_udp_thread(parse_func: fn(UdpSocket, ChannelSender), port: u16, sender: ChannelSender) -> JoinHandle<()> {
    thread::spawn(move || {
        let address = "0.0.0.0";

        let socket = match UdpSocket::bind((address, port)) {
            Ok(s) => s,
            Err(e) => panic!("couldn't bind socket: {}", e)
        };

        println!("UDP at port {}:", port);

        parse_func(socket, sender);
    })
}

const INTERVAL: Duration = Duration::from_millis(2);


fn write_every_ms(mut device: VirtualDevice) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut t0 = SystemTime::now();
        let mut delta: Duration;

        loop {
            device.write_events_from_channel_buffered().unwrap();

            match t0.elapsed() {
                Ok(passed) => {
                    delta = INTERVAL.sub(passed);
                    if delta > Duration::ZERO {
                        // println!("delta: {}", delta.as_micros());
                        sleep(delta)
                    }
                }
                Err(e) => {
                    println!("error: {}", e);
                    sleep(INTERVAL)
                }
            };
            t0 = SystemTime::now();
        }
    })
}

fn main() {
    let mut device = VirtualDevice::default().unwrap();

    create_udp_thread(parse_button, 5009, device.sender.clone());
    create_udp_thread(parse_scroll, 5007, device.sender.clone());
    create_udp_thread(parse_mouse, 5005, device.sender.clone());

    write_every_ms(device).join().unwrap();
}