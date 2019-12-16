use std::net::UdpSocket;
use std::env;
use std::men::drop;

pub struct TcpHeader{
    
}
pub struct TcpMessage {

}
fn main(){
    //Some const args(todo: set to args)
    const MWS : i32 = 16;
    const TIMEOUT : i32 = 16;
    const MSS : i32 = 128;

    //Input args <server_addr>
    let args: Vec<String> = env::args().collect();
    let server_addr : i32 = args[0].to_string();
    let local_addr= "127.0.0.1:6533".to_string();

    let local_socket = UdpSocket::bind(local_addr).except("couldn't bind to address");

    //Three-way handshake <sendSYN / waitACK / sendACK>
    local_socket.connect(server_addr).except("connect function failed");

    //Build TCP header 20-bytes
    //32-bit seq_num, 32-bit ack_num, 8-bit flag, 16-bit window size, 16-bit check_sum

    //Send data <sendData / waitACK>
    let mut seq_num : i32 = 12300;
    let mut ack_bytes : i32 = seq_num;
    let mut send_bytes : i32 = seq_num;

    loop {

    }

}