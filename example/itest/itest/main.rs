
use std::path::Path;

use itest_runner::components::container::set_up_container;
use itest_runner::components::localcli::LocalCliSetUp;
use itest_runner::components::localserver::LocalServerSetUp;
use itest_runner::components::tempdir::set_up_temp_dir;

use itest_runner::{Context, ITest, SetUpResult, itest};
use reqwest::StatusCode;
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
    assert_eq!(r#"{"message":"Hello, World!"}"#, body);
}



fn set_up_redis(ctx: &mut Context) -> SetUpResult {
    let image = GenericImage::new("redis", "7.2.4")
        .with_exposed_port(6379.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .with_network("host")
        .with_env_var("DEBUG", "1");

    set_up_container(image, ctx)
}

fn set_up_envoy(ctx: &mut Context) -> SetUpResult {
    let cfg = Path::new("../server/etc/envoy/envoy.yaml")
        .canonicalize()
        .unwrap();
    let cfg = cfg.to_str().unwrap();

    let image = GenericImage::new("envoyproxy/envoy", "v1.33-latest")
        .with_mount(
            Mount::bind_mount(cfg, "/etc/envoy/envoy.yaml")
                .with_access_mode(testcontainers::core::AccessMode::ReadOnly),
        )
        .with_network("host")
        .into();

    set_up_container(image, ctx)
}

fn set_up_postgres(ctx: &mut Context) -> SetUpResult {
    let image = GenericImage::new("postgres", "18rc1")
        .with_env_var("POSTGRES_USER", "test_user")
        .with_env_var("POSTGRES_PASSWORD", "test_password1")
        .with_env_var("POSTGRES_DB", "test_db")
        .with_network("host")
        .into();

    set_up_container(image, ctx)
}

fn set_up_schema(ctx: &mut Context) -> SetUpResult {
    LocalCliSetUp::new("example-cli")
        .with_args(&["install-schema"])
        .run(ctx)
}

fn run_server(ctx: &mut Context) -> SetUpResult {
    LocalServerSetUp::new("example-server").launch(ctx)
}

fn main() {
    ITest::new()
        .set("loglevel", "high")
        .with("cfg_dir", set_up_temp_dir)
        .with("redis", set_up_redis)
        .with("envoy", set_up_envoy)
        .with("postgres", set_up_postgres)
        .with("set-up-schema", set_up_schema)
        .with("server", run_server)
        .run();
}
