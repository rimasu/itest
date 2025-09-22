use std::{env, process::exit, time::Duration};

use clap::{Parser, Subcommand};
use sqlx::migrate;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    InstallSchema,
}

async fn create_simple_connection(db_url: &str) -> Result<sqlx::PgConnection, sqlx::Error> {
    use sqlx::Connection;
    let mut attempts = 0;
    loop {
        match sqlx::PgConnection::connect(db_url).await {
            Ok(connection) => return Ok(connection),
            Err(e) => {
                attempts += 1;
                if attempts > 20 {
                    println!("max connection failed");
                    return Err(e);
                } else {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }   
    }   
}

async fn install_schema(db_url: &str) {
    let mut pool = create_simple_connection(db_url).await.unwrap();
    migrate!("db/migrations").run(&mut pool).await.unwrap();
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let db_url = env::var("EXAMPLE_DATABASE_URL").unwrap();

    match &cli.command {
        Some(Commands::InstallSchema) => install_schema(&db_url).await,
        None => exit(-1),
    }
}
