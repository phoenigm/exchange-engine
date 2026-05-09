use std::thread;
use std::time::Duration;

fn main() {
    println!("liquidation-worker started");
    loop {
        println!("liquidation-worker heartbeat");
        thread::sleep(Duration::from_secs(5));
    }
}

