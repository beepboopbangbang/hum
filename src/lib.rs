use log::info;
use reqwest::{Error, Method, Response};
use serde_derive::Deserialize;
use serde_json::Value;
use std::{env, time::SystemTime};
use warp::Filter;

pub type ReqRes = Result<Value, reqwest::Error>;
pub type WR<T> = Result<T, warp::Rejection>;

pub const URL_API_REPOS: &str = "api/repos";
pub const URL_BUILDS: &str = "builds";
pub const RUNNING: &str = "\"running\"";

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct WebhookQuery {
    cancel_running: Option<String>,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Build {
    status: String,
    number: String,
}

pub fn api_url(base: &str, org: &str, repo: &str, num: &str) -> String {
    let mut req = vec![
        base.to_string(),
        URL_API_REPOS.to_string(),
        org.to_string(),
        repo.to_string(),
        URL_BUILDS.to_string(),
    ];

    if !num.is_empty() {
        req.push(num.to_string())
    }

    req.join("/")
}

pub fn api_builder(method: Method, org: &str, repo: &str, num: &str) -> reqwest::RequestBuilder {
    let drone_server: String = env::var("DRONE_SERVER").unwrap();
    let drone_token: String = env::var("DRONE_TOKEN").unwrap();

    let request_url = api_url(&drone_server, org, repo, num);

    info!("Sending a request to {:#?} {}", method, request_url);

    reqwest::Client::new()
        .request(method, request_url)
        .bearer_auth(drone_token)
}

pub async fn api(
    method: Method,
    org: &str,
    repo: &str,
    num: &str,
) -> Result<reqwest::Response, reqwest::Error> {
    api_builder(method, org, repo, num).send().await
}

pub async fn api_json(arb: Result<reqwest::Response, reqwest::Error>) -> ReqRes {
    let res = arb.unwrap().error_for_status()?.json::<Value>().await?;

    Ok(res)
}

pub async fn get_latest_item(org: &str, repo: &str) -> Result<Response, Error> {
    let elapsed = SystemTime::now().elapsed().unwrap();
    let arb = api(Method::GET, org, repo, &"latest".to_string()).await;

    info!(
        "Time elapsed doing latest request {:?}ms",
        elapsed.as_millis()
    );

    arb
}

pub async fn cancel_running_item(org: &str, repo: &str, num: &str) -> Result<Response, Error> {
    let elapsed = SystemTime::now().elapsed().unwrap();
    let arb = api(Method::DELETE, org, repo, num).await;

    info!(
        "Time elapsed doing cancel request {:?}ms",
        elapsed.as_millis()
    );

    arb
}

pub async fn get_latest_build(latest_res: &Value) -> Vec<String> {
    if latest_res.get("status").is_some() {
        let status = latest_res.get("status").unwrap().to_string();
        let number = latest_res.get("number").unwrap().to_string();

        vec![status, number]
    } else {
        Vec::new()
    }
}

pub async fn get_latest_and_cancel(org: &str, repo: &str) -> ReqRes {
    let elapsed = SystemTime::now().elapsed().unwrap();
    let latest_arb = get_latest_item(org, repo).await;
    let latest_res = api_json(latest_arb).await.unwrap();
    let latest_build = get_latest_build(&latest_res).await;

    if !latest_build.is_empty() && latest_build[0].eq(RUNNING) {
        info!("get_latest_item {:?}", latest_build);

        let cancel_arb = cancel_running_item(org, repo, &latest_build[1]).await;
        let cancel_res = api_json(cancel_arb).await.unwrap();

        info!(
            "cancel running build {:?}",
            cancel_res.get("number").unwrap().to_string()
        );

        info!(
            "Time elapsed doing latest & cancel request {:?}ms",
            elapsed.as_millis()
        );
    }

    Ok(latest_res)
}

pub async fn post_item(org: &str, repo: &str) -> ReqRes {
    let elapsed = SystemTime::now().elapsed().unwrap();
    let arb = api(Method::POST, org, repo, &"".to_string()).await;
    let res = api_json(arb).await?;

    info!(
        "Time elapsed doing post request {:?}ms",
        elapsed.as_millis()
    );
    Ok(res)
}

pub async fn post_render_item(
    org: String,
    repo: String,
    query: WebhookQuery,
) -> WR<impl warp::Reply> {
    if query.cancel_running.is_some() {
        let last = get_latest_and_cancel(&org, &repo).await;

        if last.is_ok() {
            let last = last.unwrap();
            let num = last.get("number");
            let stat = last.get("status");
            if num.is_some() && stat.is_some() {
                info!(
                    "latest build {:?} {:?}",
                    num.unwrap().to_string(),
                    stat.unwrap().to_string()
                );
            }
        }
    }

    let resp = post_item(&org, &repo).await;

    match resp {
        Ok(data) => Ok(warp::reply::json(&data)),
        Err(_) => Err(warp::reject()),
    }
}

pub fn post_filter() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Copy {
    warp::path::param::<String>()
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::query::<WebhookQuery>())
        .and_then(post_render_item)
}
