#![deny(warnings)]
#![cfg(not(target_arch = "wasm32"))]
mod support;
use support::*;

use dotenv::dotenv;
use hum::{api, api_json, api_url, get_latest_build, get_latest_item};
use reqwest::{Method, StatusCode};
use serde_derive::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize)]
struct HelloWorld {
    hello: String,
}

#[tokio::test]
async fn api_url_returns_correct_url() {
    let base: String = "https://test.drone.server".to_string();
    let org: String = "testing".to_string();
    let repo: String = "webhook".to_string();
    let num: String = "5".to_string();

    let mut request_url = api_url(&base, &org, &repo, "");

    let mut url: String = "https://test.drone.server/api/repos/testing/webhook/builds".to_string();

    assert_eq!(&request_url, &url);

    request_url = api_url(&base, &org, &repo, &num);
    url = [url, num].join("/");

    assert_eq!(&request_url, &url);
}

#[tokio::test]
async fn reqwest_get_latest_item() {
    dotenv().ok();
    let _ = pretty_env_logger::try_init();

    let server = server::http(move |req| async move {
        assert_eq!(req.method(), "GET");
        http::Response::new("{\"status\":\"running\",\"number\":\"123\"}".into())
    });

    env::set_var("DRONE_SERVER", format!("http://{}", server.addr()));

    let base: String = env::var("DRONE_SERVER").unwrap();
    let org: String = "testing".to_string();
    let repo: String = "webhook".to_string();

    let latest_arb = get_latest_item(&org, &repo).await;
    assert!(&latest_arb.is_ok());

    let latest_unw = latest_arb.as_ref().unwrap();
    assert_eq!(latest_unw.status(), StatusCode::OK);
    assert_eq!(
        latest_unw.url().as_str(),
        api_url(&base, &org, &repo, "latest")
    );

    let latest_res = api_json(latest_arb).await.unwrap();
    let latest_build = get_latest_build(&latest_res).await;

    assert!(!latest_build.is_empty());
    assert_eq!(latest_build[0], hum::RUNNING);
    assert_eq!(latest_build[1], "\"123\"");
}

#[tokio::test]
async fn reqwest_webhook_post() {
    dotenv().ok();
    let _ = pretty_env_logger::try_init();

    let server = server::http(move |req| async move {
        assert_eq!(req.method(), "POST");
        http::Response::new("{\"status\":\"running\",\"number\":\"123\"}".into())
    });

    env::set_var("DRONE_SERVER", format!("http://{}", server.addr()));

    let base: String = env::var("DRONE_SERVER").unwrap();
    let org: String = "testing".to_string();
    let repo: String = "webhook".to_string();

    let post_arb = api(Method::POST, &org, &repo, "").await;
    assert!(&post_arb.is_ok());

    let post_unw = post_arb.as_ref().unwrap();
    assert_eq!(post_unw.status(), StatusCode::OK);
    assert_eq!(post_unw.url().as_str(), api_url(&base, &org, &repo, ""));

    let post_res = api_json(post_arb).await;
    let post_build = get_latest_build(post_res.as_ref().unwrap()).await;

    assert!(!post_build.is_empty());
    assert_eq!(post_build[0], hum::RUNNING);
    assert_eq!(post_build[1], "\"123\"");
}
