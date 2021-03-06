extern crate dotenv;

use dotenv::dotenv;
use hum::post_filter;
use std::{env, net::IpAddr};
use warp::Filter;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let addr: IpAddr = env::var("HUM_ADDR")
        .unwrap_or_else(|_| "0.0.0.0".to_string())
        .parse()
        .unwrap();
    let port: u16 = env::var("HUM_PORT")
        .unwrap_or_else(|_| 3030.to_string())
        .parse()
        .unwrap();

    let routes = warp::any().and(post_filter());

    warp::serve(routes).run((addr, port)).await;
}
