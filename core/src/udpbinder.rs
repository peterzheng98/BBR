use std::env;
use std::process;
fn bindAddress(addr : String){
    
}

fn main(){
    let args: Vec<String> = env::args().collect();
    // Dispatch the arguments
    let address = &args[1];
    let mut mode: i8 = 0;
    if &args[2] == "server" {
        mode = 0; // indicate that this is server
        println!("Server mode! {}", args[1]);
    } else if &args[2] == "client" {
        mode = 1; // indicate that this is client
        println!("Client mode! {}", args[1]);
    } else {
        println!("Usage: ./udpbinder <ip:port> <mode>");
        process::exit(0);
    }
    // Debug output
}