use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream, UdpSocket};
use std::thread;
use std::thread::{JoinHandle, sleep};
use mouse_keyboard_input::*;
use lazy_static::lazy_static;
use bytes_convert::{first_value_from_bytes, from_bytes, to_bytes};

type CommandCodeInMsg = i8;
type AbsCoordInMsg = u16;
type RelCoordInMsg = i8;

const LEFT_MOUSE: CommandCodeInMsg = 90;
const RIGHT_MOUSE: CommandCodeInMsg = 91;
const MIDDLE_MOUSE: CommandCodeInMsg = 92;

fn to_button<S: AsRef<[u8]>>(msg: S) -> Button {
    let command_code: CommandCodeInMsg = first_value_from_bytes(msg.as_ref());
    if command_code <= 0 {
        panic!("Incorrect command code: '{}'", command_code)
    }
    match command_code {
        LEFT_MOUSE => BTN_LEFT,
        RIGHT_MOUSE => BTN_RIGHT,
        MIDDLE_MOUSE => BTN_MIDDLE,
        _ => command_code as Button,
    }
}

fn to_abs_coord<S: AsRef<[u8]>>(msg: S) -> Coord {
    let coord: AbsCoordInMsg = first_value_from_bytes(msg.as_ref());
    coord as Coord
}

fn to_rel_coord<S: AsRef<[u8]>>(msg: S) -> Coord {
    let coord: RelCoordInMsg = first_value_from_bytes(msg.as_ref());
    coord as Coord
}

fn parse_btn_press(socket: UdpSocket, sender: &ChannelSender) {
    let mut msg = [0; 1];

    loop {
        socket.recv_from(&mut msg).unwrap();
        let button = to_button(&msg);
        send_press(button, sender).unwrap();
        println!("Button pressed: {}", button);
    }
}

fn parse_btn_release(socket: UdpSocket, sender: &ChannelSender) {
    let mut msg = [0; 1];

    loop {
        socket.recv_from(&mut msg).unwrap();
        let button = to_button(&msg);
        send_release(button, sender).unwrap();
        println!("Button released: {}", button);
    }
}

fn parse_scroll(socket: UdpSocket, sender: &ChannelSender) {
    let mut msg = [0; 1];

    loop {
        socket.recv_from(&mut msg).unwrap();

        let move_by = to_rel_coord(&msg);

        send_scroll_y(move_by, sender).unwrap();
    }
}

fn parse_mouse(
    send_func: fn(coord: Coord, sender: &ChannelSender) -> EmptyResult,
    socket: UdpSocket,
    sender: &ChannelSender)
{
    let mut msg = [0; 1];

    loop {
        socket.recv_from(&mut msg).unwrap();

        let move_to = to_abs_coord(&msg);
        send_func(move_to, sender).unwrap();
    }
}

fn parse_mouse_x(socket: UdpSocket, sender: &ChannelSender) {
    parse_mouse(send_mouse_move_x, socket, sender); //FIXME:: to abs
}

fn parse_mouse_y(socket: UdpSocket, sender: &ChannelSender) {
    parse_mouse(send_mouse_move_y, socket, sender);
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


const SCREEN_SIZE_X: u32 = 1920;
const SCREEN_SIZE_Y: u32 = 1080;

lazy_static! {
    static ref SCREEN_SIZE_BYTES: Vec<u8> = to_bytes(&[SCREEN_SIZE_X, SCREEN_SIZE_Y]);
}

fn create_tcp_listener() -> JoinHandle<()> {
    thread::spawn(move || {
        let addr = format!("0.0.0.0:{}", &TCP_PC_PORT);
        let listener = TcpListener::bind(addr).unwrap();

        // accept connections and process them, spawning a new thread for each one
        println!("TCP at port {}", &TCP_PC_PORT);
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    //important not to block so android can connect 2nd time immediately
                    stream.set_nonblocking(true).unwrap();

                    let mut android_addr = stream.peer_addr().unwrap();
                    stream.write_all(SCREEN_SIZE_BYTES.as_slice()).unwrap();

                    // android_addr.set_port(TCP_ANDROID_PORT);
                    // let mut to_android_stream = TcpStream::connect(android_addr).unwrap();
                    // to_android_stream.set_nonblocking(true).unwrap();
                    // to_android_stream.write_all(SCREEN_SIZE_BYTES.as_slice()).unwrap();

                    // print in the end. It takes a long time
                    println!("New connection: {}", android_addr.ip());
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
        }
        // close the socket server
        drop(listener);
    })
}

// const WRITING_INTERVAL: Duration = Duration::from_millis(1);

type Port = u16;

const TCP_PC_PORT: Port = 5100;
// const TCP_ANDROID_PORT: Port = 5101;

const MOUSE_PORT_X: Port = 5004;
const MOUSE_PORT_Y: Port = 5005;

const SCROLL_PORT_X: Port = 5006;
const SCROLL_PORT_Y: Port = 5007;

const PRESS_BTN_PORT: Port = 5008;
const RELEASE_BTN_PORT: Port = 5009;

fn main() {
    let mut device = VirtualDevice::default().unwrap();

    create_tcp_listener();

    create_udp_thread(parse_btn_press, PRESS_BTN_PORT, device.sender.clone());
    create_udp_thread(parse_btn_release, RELEASE_BTN_PORT, device.sender.clone());
    create_udp_thread(parse_mouse_x, MOUSE_PORT_X, device.sender.clone());
    create_udp_thread(parse_mouse_y, MOUSE_PORT_Y, device.sender.clone());
    create_udp_thread(parse_scroll, SCROLL_PORT_Y, device.sender.clone());

    device.write_from_channel_every_ms();
}