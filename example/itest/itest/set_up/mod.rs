use itest_runner::set_up;

mod envoy;
mod postgres;
mod redis;
mod server;

#[set_up(Example1)]
fn set_up_example1() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
