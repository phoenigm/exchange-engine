use std::thread;
use std::time::Duration;

fn main() {
    println!("wallet-worker started");
    loop {
        println!("wallet-worker heartbeat");
        thread::sleep(Duration::from_secs(5));
    }
}
