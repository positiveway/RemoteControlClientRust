extern crate mouse_keyboard_input;

use std::net::UdpSocket;
use std::ops::{Sub};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, SystemTime};
use mouse_keyboard_input::VirtualDevice;
use mouse_keyboard_input::key_codes::*;


type SharedDevice = VirtualDevice;
type Coord = i32;
type Button = u16;
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

#[derive(Clone, Copy)]
pub struct BaseEvent {
    pub kind: u16,
    pub code: u16,
    pub value: i32,
}

type Instructions = Arc<Mutex<Vec<BaseEvent>>>;

const SYNC_EVENT: BaseEvent = BaseEvent { kind: EV_SYN, code: SYN_REPORT, value: 0 };


fn parse_button(socket: UdpSocket, instructions: Instructions) {
    let mut msg = [0; 1];
    let mut button: Button;

    loop {
        socket.recv_from(&mut msg).unwrap();
        button = msg[0] as Button;

        let mut instructions = instructions.lock().unwrap();

        if button > 128 {
            button -= 128;
            button = to_button(button);

            let event = BaseEvent { kind: EV_KEY, code: button, value: 1 };
            instructions.push(event);
            instructions.push(SYNC_EVENT);
        } else {
            button = to_button(button);

            let event = BaseEvent { kind: EV_KEY, code: button, value: 0 };
            instructions.push(event);
        }
    }
}

fn parse_scroll(socket: UdpSocket, instructions: Instructions) {
    let mut msg = [0; 1];
    let mut y: Coord;

    loop {
        socket.recv_from(&mut msg).unwrap();

        y = to_num(msg[0]);

        let event_y = BaseEvent { kind: EV_REL, code: REL_WHEEL, value: -y };

        let mut instructions = instructions.lock().unwrap();
        instructions.push(event_y);
    }
}

fn parse_mouse(socket: UdpSocket, instructions: Instructions) {
    let mut msg = [0; 2];

    let mut x: Coord;
    let mut y: Coord;

    loop {
        socket.recv_from(&mut msg).unwrap();

        x = to_num(msg[0]);
        y = to_num(msg[1]);

        let event_x = BaseEvent { kind: EV_REL, code: REL_X, value: x };
        let event_y = BaseEvent { kind: EV_REL, code: REL_Y, value: y };

        let mut instructions = instructions.lock().unwrap();

        instructions.push(event_x);
        instructions.push(event_y);
    }
}

fn create_udp_thread(parse_func: fn(UdpSocket, Instructions), port: u16, instructions: Instructions) -> JoinHandle<()> {
    thread::spawn(move || {
        let address = "0.0.0.0";

        let socket = match UdpSocket::bind((address, port)) {
            Ok(s) => s,
            Err(e) => panic!("couldn't bind socket: {}", e)
        };

        println!("UDP at port {}:", port);

        parse_func(socket, instructions);
    })
}

const INTERVAL: Duration = Duration::from_millis(5);


fn write_every_ms(instructions: Instructions) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut device = VirtualDevice::new();

        let mut t0 = SystemTime::now();
        let mut delta: Duration;

        loop {
            let mut buffer = instructions.lock().unwrap();

            buffer.push(SYNC_EVENT);

            for event in buffer.iter() {
                device.write(event.kind, event.code, event.value).unwrap();
            }

            buffer.clear();
            drop(buffer);

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
    let instructions = Arc::new(Mutex::new(Vec::new()));

    let buff1 = instructions.clone();
    create_udp_thread(parse_button, 5009, buff1);
    let buff2 = instructions.clone();
    create_udp_thread(parse_scroll, 5007, buff2);
    let buff3 = instructions.clone();
    create_udp_thread(parse_mouse, 5005, buff3);

    let buff4 = instructions.clone();
    write_every_ms(buff4).join().unwrap();
}