extern crate mouse_keyboard_input;

use std::net::UdpSocket;
use std::ops::{Sub};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, SystemTime};
use mouse_keyboard_input::{VirtualDevice, Button, Coord};
use mouse_keyboard_input::key_codes::*;


type SharedDevice = Arc<Mutex<VirtualDevice>>;
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

fn parse_button(socket: UdpSocket, device: SharedDevice) {
    let mut msg = [0; 1];
    let mut button: Button;

    loop {
        socket.recv_from(&mut msg).unwrap();
        button = msg[0] as Button;

        let mut device = device.lock().unwrap();

        if button > 128 {
            button -= 128;
            button = to_button(button);
            device.buffer_add_press(button);
        } else {
            button = to_button(button);
            device.buffer_add_release(button);
        }
    }
}

fn parse_scroll(socket: UdpSocket, device: SharedDevice) {
    let mut msg = [0; 1];
    let mut y: Coord;

    loop {
        socket.recv_from(&mut msg).unwrap();

        let mut device = device.lock().unwrap();

        y = to_num(msg[0]);

        device.buffer_add_scroll_vertical(y);
    }
}

fn parse_mouse(socket: UdpSocket, device: SharedDevice) {
    let mut msg = [0; 2];

    let mut x: Coord;
    let mut y: Coord;

    loop {
        socket.recv_from(&mut msg).unwrap();

        let mut device = device.lock().unwrap();

        x = to_num(msg[0]);
        y = to_num(msg[1]);

        device.buffer_add_mouse_move(x, y);
    }
}

fn create_udp_thread(parse_func: fn(UdpSocket, device: SharedDevice), port: u16, device: SharedDevice) -> JoinHandle<()> {
    thread::spawn(move || {
        let address = "0.0.0.0";

        let socket = match UdpSocket::bind((address, port)) {
            Ok(s) => s,
            Err(e) => panic!("couldn't bind socket: {}", e)
        };

        println!("UDP at port {}:", port);

        parse_func(socket, device);
    })
}

const INTERVAL: Duration = Duration::from_millis(2);


fn write_every_ms(device: SharedDevice) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut t0 = SystemTime::now();
        let mut delta: Duration;

        loop {
            let mut device = device.lock().unwrap();
            device.write_buffer_to_disk().unwrap();
            drop(device);

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
    let device = Arc::new(Mutex::new(VirtualDevice::new().unwrap()));

    create_udp_thread(parse_button, 5009, device.clone());
    create_udp_thread(parse_scroll, 5007, device.clone());
    create_udp_thread(parse_mouse, 5005, device.clone());

    write_every_ms(device.clone()).join().unwrap();
}