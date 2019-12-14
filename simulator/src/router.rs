use std::net::UdpSocket;
use std::env;
use std::process;
use std::str::FromStr;
use std::thread;
use std::collections;
use std::time::Duration;

// Assume this is a four-port router
// 22001 - p2p exchange
// 22002 - p2p exchange
// 22003 - client
// 22004 - server

fn main(){
    let mut arguments = Vec::new();
    for argument in env::args(){
        arguments.push(argument);
    }
    if arguments.len() != 6 {
        println!("Usage ./Router <P2P Port> <P2P Port> <Client Port> <Server Port> <Time>");
        process::exit(0);
    }

    let p2pPort1 : i32 = i32::from_str(&arguments[1]).expect("Usage ./Router <P2P Port> <P2P Port> <Client Port> <Server Port> <Time>");
    let p2pPort2 : i32 = i32::from_str(&arguments[2]).expect("Usage ./Router <P2P Port> <P2P Port> <Client Port> <Server Port> <Time>");
    let clientPort : i32 = i32::from_str(&arguments[3]).expect("Usage ./Router <P2P Port> <P2P Port> <Client Port> <Server Port> <Time>");
    let serverPort : i32 = i32::from_str(&arguments[4]).expect("Usage ./Router <P2P Port> <P2P Port> <Client Port> <Server Port> <Time>");
    let delayTime : i32 = i32::from_str(&arguments[5]).expect()
    let p2pPort1Addr = format!("127.0.0.1:{}", p2pPort1);
    let p2pPort2Addr = format!("127.0.0.1:{}", p2pPort2);
    let clientAddr = format!("127.0.0.1:{}", clientPort);
    let serverAddr = format!("127.0.0.1:{}", serverPort);
    const assumePackageSize : usize = 1024 - (1056 - 1024);
    println!("Configuration:\np2pAddr:{} <-> {}, Port:{} <-> {}\nClientAddr:{}, Port:{}\nServerAddr:{}, Port: {}", p2pPort1Addr, p2pPort2Addr, p2pPort1, p2pPort2, clientAddr, clientPort, serverAddr, serverPort);
    //Define socket here
    let routerP2PSocket1 = UdpSocket::bind(format!("127.0.0.1:{}", p2pPort1));
    let routerP2PSocket2 = UdpSocket::bind(format!("127.0.0.1:{}", p2pPort2));
    let clientSocket = UdpSocket::bind(format!("127.0.0.1:{}", clientPort));
    let serverSocket = UdpSocket::bind(format!("127.0.0.1:{}", serverPort));
    routerP2PSocket1.set_nonblocking(true).unwrap();
    routerP2PSocket2.set_nonblocking(true).unwrap();
    clientSocket.set_nonblocking(true).unwrap();
    serverSocket.set_nonblocking(true).unwrap();
    println!("Socket Bind Success!");
    println!("<-- Ready to serve -->");
    // stimulate the internal queue of the router
    let mut internalQueue : collections::VecDeque<Vec(u8)> = collections::VecDeque::new();
    let mut recvBuf1 = [0; assumePackageSize];
    let mut recvBuf2 = [0; assumePackageSize];
    let mut recvBuf3 = [0; assumePackageSize];
    let mut recvBuf4 = [0; assumePackageSize];
    loop{
        // Listen to client
        let (amt1, src1) = routerP2PSocket1.recv_from(&mut recvBuf1).unwrap();
        let (amt2, src2) = routerP2PSocket2.recv_from(&mut recvBuf2).unwrap();
        let (amtC, srcC) = clientSocket.recv_from(&mut recvBuf3).unwrap();
        let (amtS, srcS) = serverSocket.recv_from(&mut recvBuf4).unwrap();
        
    }


}