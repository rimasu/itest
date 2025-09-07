use itest_macros::itest;
use itest_runner::{SetUp, TearDown, run_all_tests};

struct RedisSetup;
struct Redis;

impl SetUp for RedisSetup {
    fn set_up(&self) -> Result<Box<dyn itest_runner::TearDown>, ()> {
        Ok(Box::new(Redis))
        // Err(())
    }

    fn name(&self) -> &str {
        "redis"
    }
}

impl TearDown for Redis {
    fn tear_down(&self) -> Result<(), ()> {
        Ok(())
    }
}

struct PostgresSetup;

struct Postgres;

impl SetUp for PostgresSetup {
    fn set_up(&self) -> Result<Box<dyn itest_runner::TearDown>, ()> {
          Ok(Box::new(Postgres))
        //Err(())
    }

    fn name(&self) -> &str {
        "postgres"
    }
}

impl TearDown for Postgres {
    fn tear_down(&self) -> Result<(), ()> {
        Ok(())
    }
}

#[itest]
fn test_two() {}

#[itest]
fn test_one_with_a_long_name() {}

fn main() {
    run_all_tests(&[Box::new(PostgresSetup), Box::new(RedisSetup)]);
}
