use exchange_engine::server::run_server;

const DEFAULT_ADDR: &str = "127.0.0.1:8080";

fn main() {
    if let Err(err) = run_server(DEFAULT_ADDR) {
        eprintln!("server failed: {err}");
        std::process::exit(1);
    }
}
