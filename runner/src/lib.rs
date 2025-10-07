#![feature(exit_status_error)]

use std::pin::Pin;
use std::{fmt, path::PathBuf, process::Command};

use async_trait::async_trait;
pub use inventory::{collect, submit};
pub use itest_macros::{depends_on, itest, set_up};

pub mod components;

mod context;
mod deptable;
mod discover;
mod single_setup_runner;
mod tasklist;

pub use context::{Context, GlobalContext, Param};

use libtest_mimic::{Arguments, Conclusion, Trial};

use crate::discover::discover_setups;
use crate::single_setup_runner::run_set_ups;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Outcome {
    Ok,
    Failed,
    Skipped,
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Outcome::Skipped => "skipped",
            Outcome::Ok => "ok",
            Outcome::Failed => "FAILED",
        })
    }
}

pub type SetFnOutput =
    Pin<Box<dyn Future<Output = Result<Option<Box<dyn TearDown>>, Box<dyn std::error::Error>>>>>;

pub type SetUpFn = fn(Context) -> SetFnOutput;

inventory::collect!(RegisteredSetUp);

pub struct RegisteredSetUp {
    pub name: &'static str,
    pub set_up_fn: SetUpFn,
    pub deps: &'static [&'static str],
    pub file: &'static str,
    pub line: usize,
}

pub struct RegisteredITest {
    pub name: &'static str,
    pub test_fn: fn(),
}
inventory::collect!(RegisteredITest);

#[async_trait]
pub trait TearDown {
    async fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

fn run_tests() -> Conclusion {
    let args = Arguments::from_args();
    let mut tests = Vec::new();

    for test in inventory::iter::<RegisteredITest> {
        tests.push(Trial::test(test.name.to_owned(), move || {
            (test.test_fn)();
            Ok(())
        }));
    }

    libtest_mimic::run(&args, tests)
}

pub struct ITest {
    context: GlobalContext,
}

impl ITest {
    pub fn new() -> Self {
        let workspace_root_dir = find_workspace_root_dir();
        let context = GlobalContext::new(&workspace_root_dir);
        Self { context }
    }
}

fn find_workspace_root_dir() -> PathBuf {
    // Get workspace root
    let output = Command::new("cargo")
        .args(&["locate-project", "--workspace", "--message-format=plain"])
        .output()
        .expect("Failed to locate workspace");

    let stdout = output.stdout;
    let workspace_root = String::from_utf8(stdout).expect("Invalid UTF-8");

    let workspace_root = workspace_root.trim().trim_end_matches("/Cargo.toml");

    PathBuf::from(workspace_root).canonicalize().unwrap()
}

impl ITest {
    pub fn set(mut self, key: &str, value: &str) -> Self {
        self.context.set_global_param(key, value);
        self
    }

    pub fn run(self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.run_async())
    }

    async fn run_async(mut self) {
        let set_ups = discover_setups().unwrap();

        let set_up_outcome = run_set_ups(set_ups, &mut self.context).await;

        let conculsion = if set_up_outcome.success {
            Some(run_tests())
        } else {
            None
        };

        let mut tear_down_result = Vec::new();
        for (name, mut tear_down) in set_up_outcome.tear_downs.into_iter().rev() {
            println!("tear down {} ", name);
            let result = (*tear_down).tear_down().await;
            tear_down_result.push(result);
        }

        if let Some(conclusion) = conculsion {
            conclusion.exit();
        }
    }
}
