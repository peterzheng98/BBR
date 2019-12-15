use std::net::UdpSocket;
use std::env;
use std::process;
use std::str::FromStr;
use std::thread;
use std::collections;
use std::time::Duration;

fn unpackHeader(header : &[u8]) -> (i32, i32, i32){
    let mut protocol = [0; 4];
    let mut port1 = [0; 4];
    let mut port2 = [0; 4];
    let mut idx : usize = 0;
    for iidx in 0..4{
        protocol[iidx] = header[idx];
        idx = idx + 1;
    }
    for iidx in 0..4{
        port1[iidx] = header[idx];
        idx = idx + 1;
    }
    for iidx in 0..4{
        port2[iidx] = header[idx];
        idx = idx + 1;
    }
    println!("Unpacking: Port and protocol is : {}<-->{}, {}, final Idx {}", i32::from_le_bytes(port1), i32::from_le_bytes(port2), i32::from_le_bytes(protocol), idx);
    (i32::from_le_bytes(protocol), i32::from_le_bytes(port1), i32::from_le_bytes(port2))
}

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
    let delayTime : i32 = i32::from_str(&arguments[5]).expect("Usage ./Router <P2P Port> <P2P Port> <Client Port> <Server Port> <Time>");
    let p2pPort1Addr = format!("127.0.0.1:{}", p2pPort1);
    let p2pPort2Addr = format!("127.0.0.1:{}", p2pPort2);
    let clientAddr = format!("127.0.0.1:{}", clientPort);
    let serverAddr = format!("127.0.0.1:{}", serverPort);
    const assumePackageSize : usize = 1024 - (1056 - 1024);
    println!("Configuration:\np2pAddr:{} <-> {}, Port:{} <-> {}\nClientAddr:{}, Port:{}\nServerAddr:{}, Port: {}", p2pPort1Addr, p2pPort2Addr, p2pPort1, p2pPort2, clientAddr, clientPort, serverAddr, serverPort);
    //Define socket here
    let routerP2PSocket1 = UdpSocket::bind(format!("127.0.0.1:{}", p2pPort1)).unwrap();
    let routerP2PSocket2 = UdpSocket::bind(format!("127.0.0.1:{}", p2pPort2)).unwrap();
    let clientSocket = UdpSocket::bind(format!("127.0.0.1:{}", clientPort)).unwrap();
    let serverSocket = UdpSocket::bind(format!("127.0.0.1:{}", serverPort)).unwrap();
    println!("Socket Bind Success!");
    println!("<-- Ready to serve -->");
    // stimulate the internal queue of the router
    let mut internalQueue : collections::VecDeque<Vec<u8>> = collections::VecDeque::new();
    let mut recvBuf1 = [0; assumePackageSize];
    let mut recvBuf2 = [0; assumePackageSize];
    let mut recvBuf3 = [0; assumePackageSize];
    let mut recvBuf4 = [0; assumePackageSize];
    let udpProtocol = i32::from_le_bytes([1,0,0,0]);
    let tcpProtocol = i32::from_le_bytes([1,1,0,0]);
    // p2pPort only accept the packet with udp
    let p2pPort1Thread = thread::spawn(move || {
        let mut packageIdx = 0;
        loop{
            let (amt, src) = routerP2PSocket1.recv_from(&mut recvBuf1).unwrap();
            let (protocol, targetPort, srcPort) = unpackHeader(&recvBuf1);
            if protocol == udpProtocol {
                routerP2PSocket1.send_to(&recvBuf1, format!("127.0.0.1:{}", targetPort));
                println!("[{}# - {}] Get packet from {:?} to port {} with size {}. Forwarded.", 1, packageIdx, src, targetPort, amt);
            } else {
                println!("[{}# - {}] Get packet not fit the header, dropped.", 1, packageIdx);
            }
            packageIdx = packageIdx + 1;
        }
    });
    let p2pPort2Thread = thread::spawn(move || {
        let mut packageIdx = 0;
        loop{
            let (amt, src) = routerP2PSocket2.recv_from(&mut recvBuf2).unwrap();
            let (protocol, targetPort, srcPort) = unpackHeader(&recvBuf2);
            if protocol == udpProtocol {
                routerP2PSocket2.send_to(&recvBuf2, format!("127.0.0.1:{}", targetPort));
                println!("[{}# - {}] Get packet from {:?} to port {} with size {}. Forwarded.", 2, packageIdx, src, targetPort, amt);
            } else {
                println!("[{}# - {}] Get packet not fit the header, dropped.", 2, packageIdx);
            }
            packageIdx = packageIdx + 1;
        }
    });
    let clientThread = thread::spawn(move || {
        let mut packageIdx = 0;
        // clientSocket.set_nonblocking(true).unwrap();
        loop{
            // Basic ideas: create a array with the incoming time.
            // the receiver side only sent the package when the corresponding time is arrived.
            // Currently this is non-waiting time version
            let (amt, src) = clientSocket.recv_from(&mut recvBuf3).unwrap();
            let (protocol, targetPort, srcPort) = unpackHeader(&recvBuf3);
            if protocol == tcpProtocol {
                clientSocket.send_to(&recvBuf3, format!("127.0.0.1:{}", targetPort));
                println!("[{}# - {}] Get packet from {:?} to port {} with size {}. Forwarded.", 3, packageIdx, src, targetPort, amt);
            } else {
                println!("[{}# - {}] Get packet not fit the header, dropped.", 3, packageIdx);
            }
            packageIdx = packageIdx + 1;
        }
    });
    let serverThread = thread::spawn(move || {
        let mut packageIdx = 0;
        // clientSocket.set_nonblocking(true).unwrap();
        loop{
            // Currently this is non-waiting time version
            let (amt, src) = serverSocket.recv_from(&mut recvBuf4).unwrap();
            let (protocol, targetPort, srcPort) = unpackHeader(&recvBuf4);
            if protocol == tcpProtocol {
                serverSocket.send_to(&recvBuf4, format!("127.0.0.1:{}", targetPort));
                println!("[{}# - {}] Get packet from {:?} to port {} with size {}. Forwarded.", 4, packageIdx, src, targetPort, amt);
            } else {
                println!("[{}# - {}] Get packet not fit the header, dropped.", 4, packageIdx);
            }
            packageIdx = packageIdx + 1;
        }
    });
    let _ = p2pPort1Thread.join();
}