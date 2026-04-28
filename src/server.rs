use crate::{
    EngineError, ExecutionReport, MatchingEngine, MarketId, OrderRequest, OrderType, Price, Quantity, Side,
    TimeInForce,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Method, Response, Server, StatusCode};

#[derive(Clone)]
struct AppState {
    engine: Arc<Mutex<MatchingEngine>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateMarketRequest {
    base: String,
    quote: String,
}

#[derive(Debug, Serialize)]
struct CreateMarketResponse {
    market: MarketDto,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MarketDto {
    base: String,
    quote: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderRequestDto {
    market: MarketDto,
    user_id: u64,
    side: String,
    order_type: String,
    price: Option<u64>,
    qty: u64,
}

#[derive(Debug, Serialize)]
struct ApiExecutionReport {
    accepted: bool,
    filled_qty: u64,
    remaining_qty: u64,
    status: String,
    last_price_after: Option<u64>,
    trades: Vec<ApiTrade>,
}

#[derive(Debug, Serialize)]
struct ApiTrade {
    price: u64,
    qty: u64,
    maker_order_id: u64,
    taker_order_id: u64,
    ts: u64,
}

#[derive(Debug, Serialize)]
struct SnapshotResponse {
    best_bid: Option<u64>,
    best_ask: Option<u64>,
    mid_price: Option<u64>,
    bids: Vec<LevelDto>,
    asks: Vec<LevelDto>,
}

#[derive(Debug, Serialize)]
struct LevelDto {
    price: u64,
    qty: u64,
}

#[derive(Debug, Serialize)]
struct LastPriceResponse {
    last_price: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn run_server(addr: &str) -> Result<(), String> {
    let state = AppState {
        engine: Arc::new(Mutex::new(MatchingEngine::new())),
    };
    let server = Server::http(addr).map_err(|e| format!("failed to bind server: {e}"))?;
    println!("exchange-engine server started at http://{addr}");
    println!("server runs continuously until terminated");

    for request in server.incoming_requests() {
        let cloned = state.clone();
        handle_request(request, cloned);
    }
    Ok(())
}

fn handle_request(mut request: tiny_http::Request, state: AppState) {
    let method = request.method().clone();
    let url = request.url().to_string();

    let response = match (method, url.as_str()) {
        (Method::Get, "/health") => json_ok(serde_json::json!({ "status": "ok" })),
        (Method::Post, "/markets") => match parse_json_body::<CreateMarketRequest>(&mut request) {
            Ok(payload) => create_market(state, payload),
            Err(err) => err,
        },
        (Method::Post, "/orders") => match parse_json_body::<OrderRequestDto>(&mut request) {
            Ok(payload) => create_order(state, payload),
            Err(err) => err,
        },
        (Method::Get, path) if path.starts_with("/markets/") && path.contains("/snapshot") => {
            get_snapshot(state, path)
        }
        (Method::Get, path) if path.starts_with("/markets/") && path.contains("/last-price") => {
            get_last_price(state, path)
        }
        _ => json_error(StatusCode(404), "not found"),
    };

    let _ = request.respond(response);
}

fn create_market(state: AppState, payload: CreateMarketRequest) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut engine = match state.engine.lock() {
        Ok(g) => g,
        Err(_) => return json_error(StatusCode(500), "engine lock poisoned"),
    };
    let market = engine.create_market(payload.base, payload.quote);
    json_ok(CreateMarketResponse {
        market: MarketDto {
            base: market.base,
            quote: market.quote,
        },
    })
}

fn create_order(state: AppState, payload: OrderRequestDto) -> Response<std::io::Cursor<Vec<u8>>> {
    let side = match parse_side(&payload.side) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let order_type = match parse_order_type(&payload.order_type) {
        Ok(v) => v,
        Err(e) => return e,
    };

    let request = OrderRequest {
        market: MarketId::new(payload.market.base, payload.market.quote),
        user_id: payload.user_id,
        side,
        order_type,
        price: payload.price.map(Price::from_raw),
        qty: Quantity::from_raw(payload.qty),
        time_in_force: Some(if order_type == OrderType::Market {
            TimeInForce::Ioc
        } else {
            TimeInForce::Gtc
        }),
    };

    let mut engine = match state.engine.lock() {
        Ok(g) => g,
        Err(_) => return json_error(StatusCode(500), "engine lock poisoned"),
    };
    match engine.submit_order(request) {
        Ok(report) => json_ok(to_api_report(report)),
        Err(err) => {
            let (status, msg) = map_engine_error(err);
            json_error(status, &msg)
        }
    }
}

fn get_snapshot(state: AppState, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let (base, quote, query) = match parse_market_path(path, "/snapshot") {
        Some(v) => v,
        None => return json_error(StatusCode(400), "invalid snapshot path"),
    };

    let depth = query
        .and_then(|q| parse_query(q).get("depth").cloned())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(5);
    let market = MarketId::new(base, quote);

    let engine = match state.engine.lock() {
        Ok(g) => g,
        Err(_) => return json_error(StatusCode(500), "engine lock poisoned"),
    };
    match engine.order_book_snapshot(&market, depth) {
        Ok(snap) => json_ok(SnapshotResponse {
            best_bid: snap.best_bid.map(|p| p.as_u64()),
            best_ask: snap.best_ask.map(|p| p.as_u64()),
            mid_price: snap.mid_price.map(|p| p.as_u64()),
            bids: snap
                .bids
                .into_iter()
                .map(|l| LevelDto {
                    price: l.price.as_u64(),
                    qty: l.qty.as_u64(),
                })
                .collect(),
            asks: snap
                .asks
                .into_iter()
                .map(|l| LevelDto {
                    price: l.price.as_u64(),
                    qty: l.qty.as_u64(),
                })
                .collect(),
        }),
        Err(err) => {
            let (status, msg) = map_engine_error(err);
            json_error(status, &msg)
        }
    }
}

fn get_last_price(state: AppState, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let (base, quote, _) = match parse_market_path(path, "/last-price") {
        Some(v) => v,
        None => return json_error(StatusCode(400), "invalid last-price path"),
    };
    let market = MarketId::new(base, quote);

    let engine = match state.engine.lock() {
        Ok(g) => g,
        Err(_) => return json_error(StatusCode(500), "engine lock poisoned"),
    };
    match engine.last_price(&market) {
        Ok(last) => json_ok(LastPriceResponse {
            last_price: last.map(|p| p.as_u64()),
        }),
        Err(err) => {
            let (status, msg) = map_engine_error(err);
            json_error(status, &msg)
        }
    }
}

fn parse_json_body<T: for<'de> Deserialize<'de>>(
    request: &mut tiny_http::Request,
) -> Result<T, Response<std::io::Cursor<Vec<u8>>>> {
    let mut body = String::new();
    if request.as_reader().read_to_string(&mut body).is_err() {
        return Err(json_error(StatusCode(400), "failed to read body"));
    }
    serde_json::from_str::<T>(&body).map_err(|_| json_error(StatusCode(400), "invalid json body"))
}

fn parse_side(s: &str) -> Result<Side, Response<std::io::Cursor<Vec<u8>>>> {
    match s.to_ascii_lowercase().as_str() {
        "bid" | "buy" => Ok(Side::Bid),
        "ask" | "sell" => Ok(Side::Ask),
        _ => Err(json_error(StatusCode(400), "invalid side")),
    }
}

fn parse_order_type(s: &str) -> Result<OrderType, Response<std::io::Cursor<Vec<u8>>>> {
    match s.to_ascii_lowercase().as_str() {
        "limit" => Ok(OrderType::Limit),
        "market" => Ok(OrderType::Market),
        _ => Err(json_error(StatusCode(400), "invalid order_type")),
    }
}

fn parse_market_path<'a>(path: &'a str, suffix: &'a str) -> Option<(&'a str, &'a str, Option<&'a str>)> {
    let (route, query) = match path.split_once('?') {
        Some((r, q)) => (r, Some(q)),
        None => (path, None),
    };
    let trimmed = route.trim_start_matches("/markets/");
    let trimmed = trimmed.strip_suffix(suffix)?;
    let mut parts = trimmed.split('/');
    let base = parts.next()?;
    let quote = parts.next()?;
    if parts.next().is_some() || base.is_empty() || quote.is_empty() {
        return None;
    }
    Some((base, quote, query))
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        }
    }
    map
}

fn map_engine_error(err: EngineError) -> (StatusCode, String) {
    match err {
        EngineError::UnknownMarket => (StatusCode(404), err.to_string()),
        EngineError::InvalidQuantity
        | EngineError::InvalidPriceForLimit
        | EngineError::PriceNotAllowedForMarket => (StatusCode(400), err.to_string()),
        EngineError::ArithmeticOverflow => (StatusCode(500), err.to_string()),
    }
}

fn to_api_report(report: ExecutionReport) -> ApiExecutionReport {
    ApiExecutionReport {
        accepted: report.accepted,
        filled_qty: report.filled_qty.as_u64(),
        remaining_qty: report.remaining_qty.as_u64(),
        status: format!("{:?}", report.status),
        last_price_after: report.last_price_after.map(|p| p.as_u64()),
        trades: report
            .trades
            .into_iter()
            .map(|trade| ApiTrade {
                price: trade.price.as_u64(),
                qty: trade.qty.as_u64(),
                maker_order_id: trade.maker_order_id,
                taker_order_id: trade.taker_order_id,
                ts: trade.ts,
            })
            .collect(),
    }
}

fn json_ok<T: Serialize>(payload: T) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{\"error\":\"serialization failed\"}".to_vec());
    Response::from_data(body)
        .with_status_code(StatusCode(200))
        .with_header(json_header())
}

fn json_error(status: StatusCode, msg: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = serde_json::to_vec(&ErrorResponse {
        error: msg.to_string(),
    })
    .unwrap_or_else(|_| b"{\"error\":\"serialization failed\"}".to_vec());
    Response::from_data(body)
        .with_status_code(status)
        .with_header(json_header())
}

fn json_header() -> Header {
    Header::from_bytes("Content-Type", "application/json").expect("invalid header")
}
