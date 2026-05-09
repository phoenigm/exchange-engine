use adapters::in_memory_catalog::InMemoryMarketCatalog;
use application::trading_app::PlaceOrderUseCase;
use domain::trading::MarketId;
use tiny_http::{Method, Response, Server, StatusCode};

fn main() {
    let server = Server::http("0.0.0.0:8080").expect("failed to bind trading-api on :8080");
    println!("trading-api listening on http://0.0.0.0:8080");

    let catalog = InMemoryMarketCatalog::new([
        "BTC/USDT".to_string(),
        "ETH/USDT".to_string(),
        "SOL/USDT".to_string(),
        "TON/USDT".to_string(),
    ]);
    let place_order = PlaceOrderUseCase::new(catalog);

    for request in server.incoming_requests() {
        let response = match (request.method(), request.url()) {
            (&Method::Get, "/health") => Response::from_string("ok"),
            (&Method::Post, "/orders/test") => {
                let market = MarketId("BTC/USDT".to_string());
                match place_order.execute(&market) {
                    Ok(()) => Response::from_string("order accepted (test)"),
                    Err(err) => Response::from_string(err).with_status_code(StatusCode(400)),
                }
            }
            _ => Response::from_string("").with_status_code(StatusCode(404)),
        };
        let _ = request.respond(response);
    }
}
