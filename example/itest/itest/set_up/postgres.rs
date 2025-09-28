use itest_runner::{AsyncSetUp, Context, components::container::set_up_container, set_up};
use testcontainers::{GenericImage, ImageExt, core::IntoContainerPort};

#[set_up(Postgres)]
fn set_up(ctx: &mut Context) -> Result<Box<dyn AsyncSetUp>, Box<dyn std::error::Error>> {
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
