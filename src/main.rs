extern crate core;

mod bytes_convert;

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream, UdpSocket};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, Instant};
use mouse_keyboard_input::*;
use bytes::{BufMut, BytesMut, Bytes};
use lazy_static::lazy_static;
use bytes_convert::*;


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
        println!("Button pressed: {}", msg[0] as Button);
    }
}

fn parse_btn_release(socket: UdpSocket, sender: &ChannelSender) {
    let mut msg = [0; 1];

    loop {
        socket.recv_from(&mut msg).unwrap();
        send_release(msg[0] as Button, sender).unwrap();
        println!("Button released: {}", msg[0] as Button);

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


const SCREEN_SIZE_X: u32 = 1080;
const SCREEN_SIZE_Y: u32 = 1920;

lazy_static!{
    static ref SCREEN_SIZE_BYTES: Vec<u8> = to_bytes(&[SCREEN_SIZE_X, SCREEN_SIZE_Y]);
}

fn handle_client(mut stream: TcpStream, screen_size: Vec<u8>) {
    let mut data = [0u8; 1];
    while match stream.read(&mut data) {
        Ok(size) => {
            stream.write(screen_size.as_slice()).unwrap();
            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn create_tcp_listener(){
    let addr = format!("0.0.0.0:{}", &TCP_PORT);
    let listener = TcpListener::bind(addr).unwrap();


    // accept connections and process them, spawning a new thread for each one
    println!("TCP Server listening on port {}", &TCP_PORT);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream, SCREEN_SIZE_BYTES.clone())
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}


const WRITING_INTERVAL: Duration = Duration::from_millis(1);

const TCP_PORT: u16 = 5100;

const MOUSE_PORT_X: u16 = 5004;
const MOUSE_PORT_Y: u16 = 5005;

const SCROLL_PORT_X: u16 = 5006;
const SCROLL_PORT_Y: u16 = 5007;

const PRESS_BTN_PORT: u16 = 5008;
const RELEASE_BTN_PORT: u16 = 5009;

fn main() {
    // let buf = to_bytes(&[-2i16, -3i16]);
    // println!("{}", buf.len());
    //
    // let c: Vec<i16> = from_bytes(buf);
    // for item in &c{
    //     println!("{}", item)
    // };

    let mut device = VirtualDevice::default().unwrap();

    create_udp_thread(parse_btn_press, PRESS_BTN_PORT, device.sender.clone());
    create_udp_thread(parse_btn_release, RELEASE_BTN_PORT, device.sender.clone());
    create_udp_thread(parse_mouse_x, MOUSE_PORT_X, device.sender.clone());
    create_udp_thread(parse_mouse_y, MOUSE_PORT_Y, device.sender.clone());
    create_udp_thread(parse_scroll, SCROLL_PORT_Y, device.sender.clone());

    device.write_from_channel_every_ms();
}