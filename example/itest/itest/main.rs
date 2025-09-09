use std::path::Path;

use itest_runner::components::container::ContainerSetUp;
use itest_runner::components::localrunner::LocalRunnerSetUp;
use itest_runner::components::tempdir::TempDirSetUp;
use itest_runner::{ITest, itest};
use reqwest::StatusCode;
use testcontainers::ContainerRequest;
use testcontainers::core::Mount;
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
};

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
    assert_eq!(
        r#"{"message":"Hello, World!"}"#,
        body
    );
}

fn set_up_redis() -> ContainerRequest<GenericImage> {
    GenericImage::new("redis", "7.2.4")
        .with_exposed_port(6379.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .with_network("host")
        .with_env_var("DEBUG", "1")
}

fn set_up_envoy() -> ContainerRequest<GenericImage> {
    let cfg = Path::new("../server/etc/envoy/envoy.yaml")
        .canonicalize()
        .unwrap();
    let cfg = cfg.to_str().unwrap();

    GenericImage::new("envoyproxy/envoy", "v1.33-latest")
        .with_mount(
            Mount::bind_mount(cfg, "/etc/envoy/envoy.yaml")
                .with_access_mode(testcontainers::core::AccessMode::ReadOnly),
        )
        .with_network("host")
        .into()
}

fn main() {
    ITest::default()
        .set("loglevel", "high")
        .with(TempDirSetUp::new("cfg_dir"))
        .with(ContainerSetUp::new(set_up_redis()))
        .with(ContainerSetUp::new(set_up_envoy()))
        .with(LocalRunnerSetUp::new("example-server"))
        .run();
}
