// extern crate async_std_hidg;
// extern crate async_std;
extern crate hidg;

// use async_std_hidg::{Class, Device, Keyboard, Key, Led, StateChange, Button, Mouse, ValueChange};
// use async_std_hidg::Key::C;
use hidg::*;
use std::io::Read;

use std::net::{UdpSocket, TcpListener, TcpStream, Shutdown};

fn to_num<T: From<i16>>(one_byte: u8) -> T {
    let mut num = one_byte as i16;
    if num > 128 {
        num -= 256;
    }
    return num.into();
}


// #[async_std::main]
// async fn main() {
fn main() {
    let mut mouse_device = Device::<Mouse>::open("hidg0").unwrap();
    // let mut mouse_device = Device::<Mouse>::open("hidg0").await.unwrap();

    let address = "0.0.0.0";
    let port = 5005;

    let socket = match UdpSocket::bind((address, port)) {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e)
    };

    println!("UDP at port {}:", port);

    let mut msg = [0; 2];

    type Coord = i16;
    let mut x: Coord;
    let mut y: Coord;

    loop {
        socket.recv_from(&mut msg).unwrap();

        x = to_num::<Coord>(msg[0]);
        y = to_num::<Coord>(msg[1]);

        let mut input = Mouse.input();

        input.change_pointer((x, y), true);

        // Send input report
        // mouse_device.input(&input).await.unwrap();
        mouse_device.input(&input).unwrap();
    }
}

fn handle_client(mut stream: TcpStream, device: &mut VirtualDevice) {
    let mut msg = [0 as u8; 1]; // using 1 byte buffer
    while match stream.read(&mut msg) {
        Ok(size) => {
            if msg[0] == 128 {
                println!("Terminating connection with {}", stream.peer_addr().unwrap());
                return;
            } else if msg[0] > 128 {
                msg[0] -= 128;
                device.press(to_button(msg[0])).unwrap();
            } else {
                device.release(to_button(msg[0])).unwrap();
            }
            true
        }
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn handle_tcp() {
    let tcp_listener = TcpListener::bind("0.0.0.0:5007").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("TCP at port 5007");

    let mut device = VirtualDevice::new();

    for stream in tcp_listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                // connection succeeded
                handle_client(stream, device.borrow_mut())
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }

    // close the socket server
    drop(tcp_listener);
}

