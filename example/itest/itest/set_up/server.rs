use itest_runner::{components::localcli::LocalCliSetUp, depends_on, set_up, GlobalContext, Context};

#[set_up(Schema)]
#[depends_on(Postgres)]
fn install_schema(ctx: Context) -> Result<(), Box<dyn std::error::Error>> {
    let db_url = ctx.get_param("Postgres.url").unwrap();
    LocalCliSetUp::new("example-cli")
        .with_args(&["install-schema"])
        .with_envs(&[("EXAMPLE_DATABASE_URL", db_url.as_str())])
        .run(ctx)
}

#[set_up(Server)]
#[depends_on(Schema)]
#[depends_on(Redis)]
fn start_server(ctx: Context) -> Result<(), Box<dyn std::error::Error>> {
    // let db_url = ctx.get_param("Postgres.url").unwrap();
    // Ok(Box::new(
    //     LocalServerSetUp::new("example-server")
    //         .with_envs(&[("EXAMPLE_DATABASE_URL", db_url.as_str())]),
    // ))
    Ok(())
}
