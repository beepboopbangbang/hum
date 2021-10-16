#![deny(warnings)]

use hum::api_url;

#[tokio::test]
async fn returns_correct_url() {
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
