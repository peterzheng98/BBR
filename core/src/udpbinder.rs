use std::env;
use std::process;
use std::net::UdpSocket;
fn listen(socket: &UdpSocket, mut buffer: &mut [u8]) -> usize{
    let (number_of_bytes, src_addr) = socket.recv_from(&mut buffer).expect("Receives nothing.");
    println!("{:?}", number_of_bytes);
    println!("{:?}", src_addr);

    number_of_bytes
}

fn send(socket: &UdpSocket, receiver: &str, msg: &Vec<u8>) -> usize{
    println!("Sending data......");
    let result = socket.send_to(msg, receiver).expect("Sending messages fail!");

    result
}

fn bindAddress(addr : String) -> UdpSocket{
    let mut socket = UdpSocket::bind(addr).expect("Cannot bind address.");
    socket
}

fn main(){
    let args: Vec<String> = env::args().collect();
    
    // Dispatch the arguments
    let address = &args[1];
    let mut mode: i8 = 0;
    let mut buf: Vec<u8> = Vec::with_capacity(65536);
    let message = String::from("abcdefghijlmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
    let message_bytes = message.into_bytes();
    if &args[2] == "server" {
        mode = 0; // indicate that this is server
        println!("Server mode! {}", args[1]);
    } else if &args[2] == "client" {
        mode = 1; // indicate that this is client
        println!("\tClient mode! {}", args[1]);
        let socket = bindAddress(String::from(address));
        loop{
            while listen(&socket, &mut buf) != 0{
                println!("Waiting....");
            }
            send(&socket, address, &message_bytes);
        }
    } else {
        println!("Usage: ./udpbinder <ip:port> <mode>");
        process::exit(0);
    }
    
}