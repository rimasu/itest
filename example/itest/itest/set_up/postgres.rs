use itest_runner::{Context, TearDown, components::container::ContainerTearDown, set_up};
use testcontainers::{GenericImage, ImageExt, core::IntoContainerPort, runners::AsyncRunner};

#[set_up(Postgres)]
async fn set_up(ctx: Context) -> Result<impl TearDown, Box<dyn std::error::Error>> {
    let image = GenericImage::new("postgres", "18rc1")
        .with_container_name("itest-postgres")
        .with_env_var("POSTGRES_USER", "test_user")
        .with_env_var("POSTGRES_PASSWORD", "test_password1")
        .with_env_var("POSTGRES_DB", "test_db")
        .with_mapped_port(15432, 5432.tcp());

    ctx.set_param(
        "url",
        "postgresql://test_user:test_password1@localhost:15432/test_db",
    );

    let container = image.start().await?;
    ctx.monitor_async("stdout", container.stdout(true));
    ctx.monitor_async("stderr", container.stderr(true));

    Ok(ContainerTearDown::new(container))
}
