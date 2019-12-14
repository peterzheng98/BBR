use std::net::UdpSocket;
use std::env;
use std::process;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

fn main(){
    let mut arguments = Vec::new();
    for argument in env::args(){
        arguments.push(argument);
    }
    
    if arguments.len() != 5 {
        println!("Usage ./p2p <Local Port> <Send Target> <Router Port> <Time>");
        process::exit(0);
    }

    let bindPort : i32 = i32::from_str(&arguments[1]).expect("Usage ./p2p <Local Port> <Send Target> <Router Port> <Time>");
    let sentPort : i32 = i32::from_str(&arguments[2]).expect("Usage ./p2p <Local Port> <Send Target> <Router Port> <Time>");
    let routerPort :i32 = i32::from_str(&arguments[3]).expect("Usage ./p2p <Local Port> <Send Target> <Router Port> <Time>");
    let time :u32 = u32::from_str(&arguments[4]).expect("Usage ./p2p <Local Port> <Send Target> <Router Port> <Time>");

    let localAddr = format!("127.0.0.1:{}", bindPort);
    let RouterInAddr = format!("127.0.0.1:{}", routerPort);
    println!("Configuration Local:{} Sent:{} Router:{} TimeInterval:{}", localAddr, sentPort, routerPort, time);
    
    const assumePackageSize : usize = 1024 - (1056 - 1024);
    let mut sentBuf = [0; assumePackageSize];
    // sentBuf[0, 1, 2, 3] indicates the protocol
    // 01 00 00 00 -> Real UDP
    // 01 01 00 00 -> Sim TCP
    sentBuf[0] = 1;
    // Here use little endian
    let mut currentIdx: usize = 4;
    let portInfo = sentPort.to_le_bytes();
    // sentBuf[4, 5, 6, 7] indicates the real target port
    for i in portInfo.iter(){
        sentBuf[currentIdx] = *i;
        currentIdx = currentIdx + 1;
    }
    // sentBuf[8, 9, 10, 11] indicates the real rawPort
    let localPortInfo = bindPort.to_le_bytes();
    println!("Bind port:{}", bindPort);
    for i in localPortInfo.iter(){
        sentBuf[currentIdx] = *i;
        currentIdx = currentIdx + 1;
    }

    let localSocket = UdpSocket::bind(localAddr).unwrap();
    let mut RecvBuf = [0; assumePackageSize];
    localSocket.send_to(&sentBuf, RouterInAddr);
    println!("Setup! <---> Ready to serve.");
    loop{
        let RouterInAddr = format!("127.0.0.1:{}", routerPort);
        let (amt, src) = localSocket.recv_from(&mut RecvBuf).unwrap();
        println!("Received {} bytes from: {:?}", amt, src);
        let mut port1 = [0; 4];
        let mut port2 = [0; 4];
        port1[0] = RecvBuf[4];
        port1[1] = RecvBuf[5];
        port1[2] = RecvBuf[6];
        port1[3] = RecvBuf[7];
        port2[0] = RecvBuf[8];
        port2[1] = RecvBuf[9];
        port2[2] = RecvBuf[10];
        port2[3] = RecvBuf[11];
        let mut unpackPort1 = i32::from_le_bytes(port1);
        let mut unpackPort2 = i32::from_le_bytes(port2);
        println!("\tUnpack Result: Target port:{}, real port:{}", unpackPort1, unpackPort2);
        localSocket.send_to(&sentBuf, RouterInAddr);
        thread::sleep(Duration::new(0, time * 1000));
    }
}