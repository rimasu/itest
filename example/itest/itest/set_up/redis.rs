use itest_runner::{AsyncSetUp, Context, components::container::set_up_container, set_up, depends_on};
use testcontainers::{GenericImage, ImageExt, core::IntoContainerPort, core::WaitFor};

#[set_up(Redis)]
fn set_up(ctx: &mut Context) -> Result<Box<dyn AsyncSetUp>, Box<dyn std::error::Error>> {
    let image = GenericImage::new("redis", "7.2.4")
        .with_exposed_port(6379.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .with_container_name("itest-redis")
        .with_env_var("DEBUG", "1");

    set_up_container(image)
}
