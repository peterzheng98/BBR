use std::net::UdpSocket;
use std::env;
use std::io;
use std::process;
use std::str::FromStr;
use std::thread;
use std::collections;
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use std::f64;

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
    if packet[12] == 1{
        ackFlag = 1;
    } else {
        ackFlag = 0;
    }
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
    let time : u32 = u32::from_str(&arguments[4]).expect("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
    let mode : i32 = i32::from_str(&arguments[5]).expect("Usage ./Client <Client Port> <Router Port> <Server Port> <Time> <Mode, 1=BBR, 2=Reno>");
    println!("Configuration:\nLocal Port:{}<->Router:{}<->Server Port:{}\nDelay Time:{} Mode,1-BBR 2-Reno: {}", clientPort, routerPort, serverPort, time, mode);
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
            cwndCount = cwnd;
            while cwndCount > 0{
                let mut sentBuf = [0; 1024];
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
                // thread::sleep(Duration::new(0, time * 1000));
                clientSocket.send_to(&sentBuf, format!("127.0.0.1:{}", routerPort));
                println!("    - Client sent package with seq {}", seqNum);
                seqNum = seqNum + 1;
                cwndCount = cwndCount - 1;
            }
            let wait_ack_timeout = SystemTime::now();
            let mut success_recv : bool = false;
            let mut duplicated : bool = false;
            let mut receivedACK : collections::BTreeMap<i32, i32> = collections::BTreeMap::new();
            let mut RecvBuf = [0; 1024];
            while wait_ack_timeout.elapsed().unwrap().as_secs() < 2 && (!success_recv) {
                // set for timeout
                let (amt, src) = clientSocket.recv_from(&mut RecvBuf).unwrap();
                let (protocol, port1, port2, ackCount, ackFlag) = unpackACK(&RecvBuf);
                if protocol == tcpProtocol{
                    if ackCount == expectedAckNum{
                        expectedAckNum = expectedAckNum + 1;
                        success_recv = true;
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
            println!("  -> Current seqNum:{}, expected ACK:{}", seqNum, expectedAckNum);
        }
        println!("Sent finished! Use time:{} seconds for 512MB", totalRunningTime.elapsed().unwrap().as_secs());
    }
    else { // TCP BBR
        println!("  * Use BBR Algorithm");
        let mut ssthresh = 128;
        let mut cwnd : i32 = 1;
        let mut cwndCount = cwnd;
        let clientSocket = UdpSocket::bind(format!("127.0.0.1:{}", clientPort)).unwrap();
        let totalRunningTime = SystemTime::now();
        let mut seqNum : i32 = 0;
        let mut expectedAckNum : i32 = 0;


        //bw and inflight -> based on packet num, not size
        let startup_pace : f64 = 2885.0 / 1000.0;
        let drain_pace : f64= 1000.0 / 2885.0;
        let bw_pacing_gain = [5.0 / 4.0, 3.0 / 4.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let rtt_pace:f64 = 1.0;
        let mut state:i32 = 0; // 0: startup | 1: drain | probe_bw | probe_rtt
        let mut cur_bw_pace_index : usize = 0;

        let mut BtlBw_max : f64 = 0.0;
        let mut RTprop_min : u128 = std::u128::MAX;

//        let mut last_ack_time : collections::BTreeMap<i32, i32> = collections::BTreeMap::new();
        let mut sendTime : collections::HashMap<i32, SystemTime> = HashMap::new();
        let mut delivered_num : collections::HashMap<i32, i32> = HashMap::new();
        let mut inflight_num :i32 = 0;
        let mut inflight : f64 = 0.0;
        let mut sent_packet_num : i32 = 0;


        let mut rtt_round : bool = false;
        let mut rtt_round_seq_num: i32 = seqNum;
        let mut last_round_bw: f64 = 0.0;
        let mut cur_time : SystemTime = SystemTime::now();
        let mut sent_time : SystemTime = SystemTime::now();

        let mut rtt_modify_time : SystemTime = SystemTime::now();
        let mut bw_low_increase_cnt : i32 = 0; //maximum = 3

        let mut nextSendTime : SystemTime = SystemTime::now();
        let mut sendMargin : u128 = 0;

        while expectedAckNum < 512 * 1024{
            println!("-------------In a new Round----------");
            // sent packet with
            let localPortInfo = clientPort.to_le_bytes();
            let serverPortInfo = serverPort.to_le_bytes();
            cwndCount = cwnd;
            println!("  Current cwnd is {}", cwnd);
            let mut cur_bdp : f64 = BtlBw_max  * (RTprop_min as f64);
            println!("  Current_State -> {}", state);
            println!("  Current_BDP -> {}", cur_bdp);
            while cwndCount > 0{
                inflight = inflight_num as f64 * 1024.0;
                cur_time = SystemTime::now();
                if cur_time > nextSendTime {
                    let mut sentBuf = [0; 1024];
                    let seqInfo = seqNum.to_le_bytes();
                    // TCP Header
                    sentBuf[0] = 1;
                    sentBuf[1] = 1;

                    // Target Port
                    let mut currentIdx = 4;
                    for i in serverPortInfo.iter() {
                        sentBuf[currentIdx] = *i;
                        currentIdx = currentIdx + 1;
                    }
                    // Sent Port
                    for i in localPortInfo.iter() {
                        sentBuf[currentIdx] = *i;
                        currentIdx = currentIdx + 1;
                    }
                    // Set Seq Flag = 1
                    sentBuf[14] = 1;
                    // Set Sent Seq Count
                    currentIdx = 20;
                    for i in seqInfo.iter() {
                        sentBuf[currentIdx] = *i;
                        currentIdx = currentIdx + 1;
                    }
                    // thread::sleep(Duration::new(0, time * 1000));
                    clientSocket.send_to(&sentBuf, format!("127.0.0.1:{}", routerPort));
                    sent_time = SystemTime::now();
                    inflight_num = inflight_num + 1;
                    sent_packet_num = sent_packet_num + 1;
                    sendTime.insert(seqNum, sent_time);
                    delivered_num.insert(seqNum, sent_packet_num);
                    nextSendTime = sent_time + Duration::from_nanos(sendMargin as u64);

                    if !rtt_round{
                        rtt_round_seq_num = seqNum;
                        rtt_round = true;
                    }

//                    println!("    - Client sent package with seq {}", seqNum);
                    seqNum = seqNum + 1;
                    cwndCount = cwndCount - 1;
                }
            }
            let wait_ack_timeout = SystemTime::now();
            let mut success_recv : bool = false;
            let mut duplicated : bool = false;
            let mut receivedACK : collections::BTreeMap<i32, i32> = collections::BTreeMap::new();
            let mut RecvBuf = [0; 1024];
            while wait_ack_timeout.elapsed().unwrap().as_secs() < 2 && (!success_recv) {
                // set for timeout
                let (amt, src) = clientSocket.recv_from(&mut RecvBuf).unwrap();
                let (protocol, port1, port2, ackCount, ackFlag) = unpackACK(&RecvBuf);
                if protocol == tcpProtocol{

                    let correspond_ackTime : SystemTime = *sendTime.get(&ackCount).unwrap();//todo
                    let correspond_ackDuration : u128 = correspond_ackTime.elapsed().unwrap().as_nanos();

                    let mut cur_rtt : u128 = correspond_ackDuration; //unit: ns
                    let mut inflight_margin : i32 = sent_packet_num - *delivered_num.get(&ackCount).unwrap() + 1;
                    inflight_num = inflight_num - 1;

                    let mut cur_bw : f64 = inflight_margin as f64 * 1024.0 / cur_rtt as f64; // bytes/ns

                    if cur_bw > BtlBw_max{
                        BtlBw_max = cur_bw;
                    }
                    let rtt_modify_delta: u128 = rtt_modify_time.elapsed().unwrap().as_nanos();
                    if rtt_modify_delta > 10 * 1000000000{
                        rtt_modify_time = SystemTime::now();
                        RTprop_min = cur_rtt;
                        state = 3;
                    }
                    else if cur_rtt < RTprop_min && !(state == 3){
                        rtt_modify_time = SystemTime::now();
                        RTprop_min = cur_rtt;
                    }

                    let mut pace_rate : f64 = 0.0;
                    let mut cur_bdp = BtlBw_max * (RTprop_min as f64);
                    match state {
                        0 => {
                            if ackCount == rtt_round_seq_num && rtt_round{
                                if cur_bw < (1.0 + 0.25) * last_round_bw{
                                    bw_low_increase_cnt = bw_low_increase_cnt + 1;
                                }
                                last_round_bw = cur_bw;
                                rtt_round = false;
                            }
                            if bw_low_increase_cnt > 3{
                                pace_rate = drain_pace;
                                state = 1;
                                bw_low_increase_cnt = 0;
                            }
                            else {
                                pace_rate = startup_pace;
                            }
                            cwnd = (cwnd as f64 * pace_rate) as i32;
                        },
                        1 => {
                            inflight = inflight_num as f64 * 1024.0;
                            if cur_bdp >= inflight{
                                state = 2;
                                pace_rate = bw_pacing_gain[cur_bw_pace_index];
                                cur_bw_pace_index = (cur_bw_pace_index + 1) % 8;
                            }
                            else{
                                pace_rate = drain_pace;
                            }
                            cwnd = (cwnd as f64 * pace_rate) as i32;
                        },

                        2 => {
                            inflight = inflight_num as f64 * 1024.0;
                            pace_rate = bw_pacing_gain[cur_bw_pace_index];
                            let mut next_pace : bool = false;
                            if pace_rate == 1.0{
                                if cur_rtt > RTprop_min{
                                    next_pace = true;
                                }

                            }
                            else if pace_rate > 1.0{
                                if cur_rtt > RTprop_min && inflight >= cur_bdp{
                                    next_pace = true;
                                }
                            }
                            else{
                                if cur_rtt > RTprop_min || inflight <= cur_bdp{
                                    next_pace = true;
                                }

                            }
                            if next_pace {
                                cur_bw_pace_index = (cur_bw_pace_index + 1) % 8;
                            }

                            cwnd = 2 * cwnd;
                        },

                        3 => {
                            cwnd = 4;
                            if SystemTime::now().duration_since(rtt_modify_time).expect("Time Error").as_micros() > 200{
                                state = 0;
                            }
                            pace_rate = rtt_pace;
                        },

                        _ =>{
                            break;
                        },

                    };
                    sendMargin = (1024.0 / (pace_rate * BtlBw_max)) as u128 ;
                    if ackCount == expectedAckNum{
                        expectedAckNum = expectedAckNum + 1;
                    }
                    if expectedAckNum == seqNum{
                        success_recv = true;
                    }
                }
            }
            println!("  -> Current seqNum:{}, expected ACK:{}", seqNum, expectedAckNum);
        }
        println!("Sent finished! Use time:{} seconds for 512MB", totalRunningTime.elapsed().unwrap().as_secs());
    }


}