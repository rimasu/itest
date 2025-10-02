use itest_runner::components::localserver::LocalServerSetUp;
use itest_runner::components::localcli::LocalCliSetUp;
use itest_runner::{AsyncSetUp, Context, depends_on, set_up};


#[set_up(Schema)]
#[depends_on(Postgres)]
fn install_schema(ctx: &mut Context) -> Result<Box<dyn AsyncSetUp>, Box<dyn std::error::Error>> {
    let db_url = ctx.get_param("Postgres.url").unwrap();
    Ok(Box::new(
        LocalCliSetUp::new("example-cli")
            .with_args(&["install-schema"])
            .with_envs(&[("EXAMPLE_DATABASE_URL", db_url.as_str())]),
    ))
}


#[set_up(Server)]
#[depends_on(Schema)]
#[depends_on(Redis)]
fn start_server(ctx: &mut Context) -> Result<Box<dyn AsyncSetUp>, Box<dyn std::error::Error>> {
    let db_url = ctx.get_param("Postgres.url").unwrap();
    Ok(Box::new(
        LocalServerSetUp::new("example-server")
            .with_envs(&[("EXAMPLE_DATABASE_URL", db_url.as_str())]),
    ))
}
