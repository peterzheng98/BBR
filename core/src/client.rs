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

fn unpackACK(packet : &[u8]) -> (i32, i32, i32, i32, i32){
    let mut protocol = [0; 4];
    let mut port1 = [0; 4];
    let mut port2 = [0; 4];
    let mut ackFlag : i32 = 0;
    let mut ackCount = [0; 4];
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
    idx = 16;
    for iidx in 0..4{
        ackCount[iidx] = packet[idx];
        idx = idx + 1;
    }
    ackFlag = packet[12];
    println!("    = Unpacking ACK: Port and protocol: {} <--> {}, {}, ackflag {} acknum {}", i32::from_le_bytes(port1), i32::from_le_bytes(port2), i32::from_le_bytes(protocol), ackFlag, i32::from_le_bytes(ackCount));
    (i32::from_le_bytes(protocol), i32::from_le_bytes(port1), i32::from_le_bytes(port2), i32::from_le_bytes(ackCount), ackFlag)
}

fn main(){
    let mut arguments = Vec::new();
    for argument in env::args(){
        arguments.push(argument);
    }
    if arguments.len() != 6 {
        println!("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
        process::exit(0);
    }
    let udpProtocol = i32::from_le_bytes([1,0,0,0]);
    let tcpProtocol = i32::from_le_bytes([1,1,0,0]);

    let clientPort : i32 = i32::from_str(&arguments[1]).expect("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
    let routerPort : i32 = i32::from_str(&arguments[2]).expect("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
    let serverPort : i32 = i32::from_str(&arguments[3]).expect("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
    let time : i32 = i32::from_str(&arguments[4]).expect("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
    let mode : i32 = i32::from_str(&arguments[5]).expect("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
    println!("Configuration:\nLocal Port:{}<->Router:{}<->Server Port:{}\nDelay Time:{} Mode,1-BBR 2-Reno{}", clientPort, routerPort, serverPort, time, mode);
    println!("<-- Ready to run, Type any character for continuing -->");
    let mut inputControl = String::new();
    io::stdin().read_line(&mut inputControl).expect("IOError");
    const totalSize : i128 = 512 * 1024 * 1024;
    let mut currentSentSize = 0;
    println!("  * Required sent size {} bytes, 1K bytes per packet. Assume waiting time is the same {} milliseconds.", totalSize, time);
    if mode == 2{ // Use reno
        println!("  * Use Reno Algorithm");
        let mut ssthresh = 128;
        let mut cwnd = 1;
        let mut cwndCount = cwnd;
        let clientSocket = UdpSocket::bind(format!("127.0.0.1:{}", clientPort)).unwrap();
        let totalRunningTime = SystemTime::now();
        let mut seqNum : i32 = 0;
        let mut expectedAckNum : i32 = 0;
        while expectedAckNum < 512 * 1024{
            // sent packet with 
            let localPortInfo = clientPort.to_le_bytes();
            let serverPortInfo = serverPort.to_le_bytes();
            while cwndCount > 0{
                let mut sentBuf = [0; (1024 + 32)];

                let seqInfo = seqNum.to_le_bytes();
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
                sentBuf[14] = 1;
                // Set Sent Seq Count
                currentIdx = 20;
                for i in seqInfo.iter(){
                    sentBuf[currentIdx] = *i;
                    currentIdx = currentIdx + 1;
                }
                thread::sleep(Duration::new(0, time * 1000));
                clientSocket.send_to(&sentBuf, format!("127.0.0.1:{}", routerPort));
                println!("    - Client sent package with seq {}", seqNum);
                seqNum = seqNum + 1;
                cwndCount = cwndCount - 1;
            }
            let wait_ack_timeout = SystemTime::now();
            let mut success_recv : bool = false;
            let mut duplicated : bool = false;
            let mut receivedACK : collections::BTreeMap<i32, i32> = collections::BTreeMap::new();
            let mut RecvBuf = [0; (1024 + 32)];
            while wait_ack_timeout.elapsed().unwrap().as_secs() < 5 && (!success_recv) {
                // set for timeout
                let (amt, src) = clientPort.recv_from(&mut RecvBuf).unwrap();
                let (protocol, port1, port2, ackCount, ackFlag) = unpackACK(&RecvBuf);
                if protocol == tcpProtocol{
                    if ackCount == expectedAckNum{
                        expectedAckNum = expectedAckNum + 1;
                    } else {
                        if !receivedACK.contains_key(&ackCount) {
                            receivedACK.insert(ackCount, 1);
                        } else {
                            // the key exists
                            if let Some(x) = receivedACK.get_mut(&ackCount) {
                                *x += 1;
                                if *x == 3{
                                    duplicated = true;
                                    success_recv = true;
                                    seqNum = ackCount;
                                }
                            }
                        }
                        while receivedACK.contains_key(&expectedAckNum) {expectedAckNum = expectedAckNum + 1;}
                    }
                    if expectedAckNum == seqNum - 1{
                        success_recv = true;
                        if cwnd > ssthresh{
                            cwnd = cwnd + 1; // TCP Congestion Control
                        } else {
                            cwnd = cwnd * 2; // TCP Slow Start
                        }
                        println!("    - Client accept correct ACK, update cwnd");
                    }
                }
            }
            // modify the cwnd
            // if timeout -> ssthresh = cwnd / 2, cwnd = 1
            // if duplicated -> ssthresh = cwnd / 2, cwnd = ssthresh
            if (!success_recv) && (!duplicated) {
                ssthresh = cwnd / 2;
                cwnd = 1;
                println!("  ! Timeout detected, add ssthresh {}, cwnd {}", ssthresh, cwnd);
            }
            if (success_recv) && (duplicated) {
                ssthresh = cwnd / 2;
                cwnd = ssthresh;
                println!("  ! 3 DUP ACK detected, add ssthresh {}, cwnd {}", ssthresh, cwnd);
            }
            println!("  ->Current seqNum:{}, expected ACK:{}", seqNum, expectedAckNum);
        }
        println!("Sent finished! Use time:{} seconds for 512MB", totalRunningTime.elapsed().unwrap().as_secs());
    } else { // TCP BBR

    }



}