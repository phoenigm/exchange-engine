use std::thread;
use std::time::Duration;

fn main() {
    println!("risk-worker started");
    loop {
        println!("risk-worker heartbeat");
        thread::sleep(Duration::from_secs(5));
    }
}

