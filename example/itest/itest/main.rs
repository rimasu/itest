use std::path::Path;

use itest_runner::components::container::set_up_container;
use itest_runner::components::localcli::LocalCliSetUp;
use itest_runner::components::localserver::LocalServerSetUp;

use itest_runner::components::tempdir::set_up_temp_dir;
use itest_runner::{AsyncSetUp, Context, ITest, depends_on, itest, set_up};
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

#[set_up(Redis)]
fn set_up_redis(ctx: &mut Context) -> Result<Box<dyn AsyncSetUp>, Box<dyn std::error::Error>> {
    let image = GenericImage::new("redis", "7.2.4")
        .with_exposed_port(6379.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .with_container_name("itest-redis")
        .with_env_var("DEBUG", "1");

    set_up_container(image)
}

#[set_up(Envoy)]
fn set_up_envoy(ctx: &mut Context) -> Result<Box<dyn AsyncSetUp>, Box<dyn std::error::Error>> {
    let cfg = Path::new("../server/etc/envoy/envoy.yaml")
        .canonicalize()
        .unwrap();
    let cfg = cfg.to_str().unwrap();

    let image = GenericImage::new("envoyproxy/envoy", "v1.33-latest")
        .with_container_name("itest-envoy")
        .with_mount(
            Mount::bind_mount(cfg, "/etc/envoy/envoy.yaml")
                .with_access_mode(testcontainers::core::AccessMode::ReadOnly),
        )
        .with_network("host")
        .into();

    set_up_container(image)
}

#[depends_on(Postgres)]
#[set_up(Postgres)]
fn set_up_postgres(ctx: &mut Context) -> Result<Box<dyn AsyncSetUp>, Box<dyn std::error::Error>> {
    let image = GenericImage::new("postgres", "18rc1")
        .with_container_name("itest-postgres")
        .with_env_var("POSTGRES_USER", "test_user")
        .with_env_var("POSTGRES_PASSWORD", "test_password1")
        .with_env_var("POSTGRES_DB", "test_db")
        .with_mapped_port(15432, 5432.tcp())
        .into();

    ctx.set_param(
        "url",
        "postgresql://test_user:test_password1@localhost:15432/test_db",
    );

    set_up_container(image)
}

#[set_up(Schema)]
#[depends_on(Postgres)]
#[depends_on(Postgres)]
fn install_schema(ctx: &mut Context) -> Result<Box<dyn AsyncSetUp>, Box<dyn std::error::Error>> {
    let db_url = ctx.get_param("postgres.url").unwrap();
    Ok(Box::new(
        LocalCliSetUp::new("example-cli")
            .with_args(&["install-schema"])
            .with_envs(&[("EXAMPLE_DATABASE_URL", db_url.as_str())]),
    ))
}

#[set_up(Server)]
#[depends_on(Postgres)]
fn start_server(ctx: &mut Context) -> Result<Box<dyn AsyncSetUp>, Box<dyn std::error::Error>> {
    let db_url = ctx.get_param("postgres.url").unwrap();
    Ok(Box::new(
        LocalServerSetUp::new("example-server")
            .with_envs(&[("EXAMPLE_DATABASE_URL", db_url.as_str())]),
    ))
}

fn main() {
    ITest::new()
        .set("loglevel", "high")
        // .with("cfg_dir", set_up_temp_dir)
        // .with("other_dir", set_up_temp_dir)
        // .with("redis", set_up_redis)
        // .with("envoy", set_up_envoy)
        // .with("postgres", set_up_postgres)
        // .with("schema", {
        //     |ctx| {
        //         let db_url = ctx.get_param("postgres.url").unwrap();
        //         Ok(Box::new(
        //             LocalCliSetUp::new("example-cli")
        //                 .with_args(&["install-schema"])
        //                 .with_envs(&[("EXAMPLE_DATABASE_URL", db_url.as_str())]),
        //         ))
        //     }
        // })
        // .with("server", {
        //     |ctx| {
        //         let db_url = ctx.get_param("postgres.url").unwrap();
        //         Ok(Box::new(
        //             LocalServerSetUp::new("example-server")
        //                 .with_envs(&[("EXAMPLE_DATABASE_URL", db_url.as_str())]),
        //         ))
        //     }
        // })
        .run();
}
