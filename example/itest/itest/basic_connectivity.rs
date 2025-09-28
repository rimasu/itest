use itest_runner::itest;
use reqwest::StatusCode;

#[itest]
fn can_not_call_server_directly_with_http1() {
    let response = reqwest::blocking::get("http://localhost:3000/").unwrap();
    assert_eq!(StatusCode::HTTP_VERSION_NOT_SUPPORTED, response.status());
    let body = response.text().unwrap();
    assert_eq!(
        r#"{"error":"This server only accepts HTTP/2 connections","received_version":"HTTP/1.1"}"#,
        body
    );
}

#[itest]
fn can_call_server_via_envoy_with_http1() {
    let response = reqwest::blocking::get("http://localhost:8080/").unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = response.text().unwrap();
    assert_eq!(r#"{"message":"Hello, World!"}"#, body);
}
