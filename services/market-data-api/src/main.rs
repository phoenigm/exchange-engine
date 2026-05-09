use tiny_http::{Method, Response, Server, StatusCode};

fn main() {
    let server = Server::http("0.0.0.0:8081").expect("failed to bind market-data-api on :8081");
    println!("market-data-api listening on http://0.0.0.0:8081");

    for request in server.incoming_requests() {
        let response = match (request.method(), request.url()) {
            (&Method::Get, "/health") => Response::from_string("ok"),
            (&Method::Get, "/ticker/BTCUSDT") => {
                Response::from_string(r#"{"symbol":"BTCUSDT","last":"0","bid":"0","ask":"0"}"#)
            }
            _ => Response::from_string("").with_status_code(StatusCode(404)),
        };
        let _ = request.respond(response);
    }
}
