use itest_runner::components::container::ContainerSetUp;
use itest_runner::components::tempdir::TempDirSetUp;
use itest_runner::{ITest, itest};
use testcontainers::ContainerRequest;
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
};

#[itest]
fn test_two() {}

#[itest]
fn test_one_with_a_long_name() {}

fn set_up_redis1() -> ContainerRequest<GenericImage> {
    GenericImage::new("redis", "7.2.4")
        .with_exposed_port(6379.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .with_network("bridge")
        .with_env_var("DEBUG", "1")
}

fn set_up_redis2() -> ContainerRequest<GenericImage> {
    GenericImage::new("redis", "7.2.4")
        .with_exposed_port(6379.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .with_network("bridge")
        .with_env_var("DEBUG", "1")
}

fn main() {
    ITest::default()
        .set("loglevel", "high")
        .with(TempDirSetUp::new("cfg_dir"))
        .with(ContainerSetUp::new(set_up_redis1()))
        .with(ContainerSetUp::new(set_up_redis2()))
        .run();
}
