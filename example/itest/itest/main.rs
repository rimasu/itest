use itest_runner::ITest;

mod basic_connectivity;
mod set_up;

fn main() {
    ITest::new().set("loglevel", "high").run();
}
