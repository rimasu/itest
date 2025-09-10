use std::{process::exit, time::Duration};

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

async fn create_simple_connection() -> Result<sqlx::PgConnection, sqlx::Error> {
    use sqlx::Connection;
    let mut attempts = 0;
    loop {
        let database_url = "postgresql://test_user:test_password1@localhost:5432/test_db";
        match sqlx::PgConnection::connect(database_url).await {
            Ok(connection) => return Ok(connection),
            Err(e) => {
                attempts += 1;
                if attempts > 20 {
                    println!("max connectiont failed");
                    return Err(e);
                } else {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }   
    }
}

async fn install_schema() {
    let mut pool = create_simple_connection().await.unwrap();
    migrate!("db/migrations").run(&mut pool).await.unwrap();
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::InstallSchema) => install_schema().await,
        None => exit(-1),
    }
}
