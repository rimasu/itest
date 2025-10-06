use itest_runner::{Context, TearDown, components::container::ContainerTearDown, set_up};

use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};

#[set_up(Redis)]
async fn set_up(ctx: Context) -> Result<impl TearDown, Box<dyn std::error::Error>> {
    let image = GenericImage::new("redis", "7.2.4")
        .with_exposed_port(6379.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .with_container_name("itest-redis")
        .with_env_var("DEBUG", "1");

    let container = image.start().await?;

    ctx.monitor_async("stdout", container.stdout(true));
    ctx.monitor_async("stderr", container.stderr(true));

    Ok(ContainerTearDown::new(container, &ctx))
}
