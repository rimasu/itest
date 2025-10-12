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
mod set_up_runner;
mod tasklist;

mod set_up_workers;
mod progress;

pub use context::{Context, GlobalContext, Param};

use libtest_mimic::{Arguments, Conclusion, Trial};

use crate::discover::discover_setups;
use crate::progress::launch_progress_monitor;
use crate::set_up_runner::run_set_ups;




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

#[derive(Debug)]
pub enum SetUpError {
    Generic(String),
}

impl From<Box<dyn std::error::Error>> for SetUpError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        SetUpError::Generic(format!("{}", value))
    }
}

pub type SetUpResult = Result<Option<Box<dyn TearDown>>, SetUpError>;

pub type SetFnOutput = Pin<Box<dyn Future<Output = SetUpResult> + Send +'static>>;

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
pub trait TearDown: Send {
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

pub struct ITest {}

impl ITest {
    pub fn new() -> Self {
        Self {}
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
    pub fn run(self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.run_async())
    }

    async fn run_async(self) {
        let workspace_root_dir = find_workspace_root_dir();

        let set_ups = discover_setups().unwrap();
        let task_names = set_ups.tasks().map(|(t, n)| (t, n.to_string())).collect();

        let (_, listener) = launch_progress_monitor(task_names);
        let mut context = GlobalContext::new(&workspace_root_dir);

        let set_up_outcome = run_set_ups(set_ups, &mut context, listener).await;

        let conculsion = if set_up_outcome.success {
            Some(run_tests())
        } else {
            None
        };

        let mut tear_down_result = Vec::new();
        for (name, mut tear_down) in set_up_outcome.tear_downs.into_iter().rev() {
            println!("tear down {} ", name.0);
            let result = (*tear_down).tear_down().await;
            tear_down_result.push(result);
        }

        if let Some(conclusion) = conculsion {
            conclusion.exit();
        }
    }
}
