use std::net::UdpSocket;
use std::env;
use std::io;
use std::process;
use std::str::FromStr;
use std::thread;
use std::collections;
use std::time::{Duration, SystemTime};

// Usage: <Client Port> <Router Port> <Server Port> <Time> <Mode>
// Sim-TCP Header 01 01 00 00 - TCP
//                .. .. .. .. - Port 1
//                .. .. .. .. - Port 2
//                00 - Basic Control Information

fn packACK(ackNum: i32, target: i32, src: i32) -> [u8; (1024 + 30)]{
    let mut sentBuf = [0; (1024 + 30)];
    let seqInfo = ackNum.to_le_bytes();
    let serverPortInfo = target.to_le_bytes();
    let localPortInfo = src.to_le_bytes();
    // TCP Header
    sentBuf[0] = 1;
    sentBuf[1] = 1;
    // Target Port
    let mut currentIdx = 4;
    for i in serverPortInfo.iter(){
        sentBuf[currentIdx] = *i;
        currentIdx = currentIdx + 1;
    }
    // Sent Port
    for i in localPortInfo.iter(){
        sentBuf[currentIdx] = *i;
        currentIdx = currentIdx + 1;
    }
    // Set Seq Flag = 1
    sentBuf[12] = 1;
    // Set Sent Seq Count
    currentIdx = 16;
    for i in seqInfo.iter(){
        sentBuf[currentIdx] = *i;
        currentIdx = currentIdx + 1;
    }
    sentBuf
}

fn unpackSeq(packet : &[u8]) -> (i32, i32, i32, i32, i32){
    let mut protocol = [0; 4];
    let mut port1 = [0; 4];
    let mut port2 = [0; 4];
    let mut seqFlag : i32 = 0;
    let mut seqCount = [0; 4];
    let mut idx : usize = 0;
    for iidx in 0..4{
        protocol[iidx] = packet[idx];
        idx = idx + 1;
    }
    for iidx in 0..4{
        port1[iidx] = packet[idx];
        idx = idx + 1;
    }
    for iidx in 0..4{
        port2[iidx] = packet[idx];
        idx = idx + 1;
    }
    idx = 20;
    for iidx in 0..4{
        seqCount[iidx] = packet[idx];
        idx = idx + 1;
    }
    if packet[14] == 1{
        seqFlag = 1;
    } else {
        seqFlag = 0;
    }
    println!("    = Unpacking Seq: Port and protocol: {} <--> {}, {}, ackflag {} acknum {}", i32::from_le_bytes(port1), i32::from_le_bytes(port2), i32::from_le_bytes(protocol), seqFlag, i32::from_le_bytes(seqCount));
    (i32::from_le_bytes(protocol), i32::from_le_bytes(port1), i32::from_le_bytes(port2), i32::from_le_bytes(seqCount), seqFlag)
}

fn main(){
    let mut arguments = Vec::new();
    for argument in env::args(){
        arguments.push(argument);
    }
    if arguments.len() != 6 {
        println!("Usage ./Server <Server Port> <Router Port> <Client Port> <Time> <Mode, 1=BBR, 2=Reno>");
        process::exit(0);
    }
    let udpProtocol = i32::from_le_bytes([1,0,0,0]);
    let tcpProtocol = i32::from_le_bytes([1,1,0,0]);

    let clientPort : i32 = i32::from_str(&arguments[3]).expect("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
    let routerPort : i32 = i32::from_str(&arguments[2]).expect("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
    let serverPort : i32 = i32::from_str(&arguments[1]).expect("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
    let time : i32 = i32::from_str(&arguments[4]).expect("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
    let mode : i32 = i32::from_str(&arguments[5]).expect("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
    println!("Configuration:\nLocal Port:{}<->Router:{}<->Server Port:{}\nDelay Time:{} Mode,1-BBR 2-Reno{}", clientPort, routerPort, serverPort, time, mode);
    println!("<-- Ready to run, Type any character for continuing -->");
    let mut inputControl = String::new();
    io::stdin().read_line(&mut inputControl).expect("IOError");
    let serverSocket = UdpSocket::bind(format!("127.0.0.1:{}", serverPort)).unwrap();
    let mut expectSeq : i32 = 0;
    let mut ackNum : i32 = 0;
    let mut RecvBuf = [0; (1024 + 32)];
    if mode == 2{
        loop{
            let (amt, src) = serverSocket.recv_from(&mut RecvBuf).unwrap();
            let (protocol, port1, port2, seqCount, _) = unpackSeq(&RecvBuf);
            println!("  - Server receives {}->{} seqCount:{} realSrc:{:?}", port2, port1, seqCount, src);
            let mut sentbuffer = packACK(seqCount, port2, port1);
            serverSocket.send_to(&sentbuffer, format!("127.0.0.1:{}", routerPort));
        }
    }
}