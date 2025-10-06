use itest_runner::set_up;

mod envoy;
mod postgres;
mod redis;
mod server;

#[set_up(Example)]
fn set_up() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
