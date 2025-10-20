#![feature(exit_status_error)]
#![feature(exitcode_exit_method)]

use std::pin::Pin;
use std::process::ExitCode;
use std::{path::PathBuf, process::Command};

use async_trait::async_trait;
pub use inventory::{collect, submit};
pub use itest_macros::{depends_on, itest, set_up};

pub mod components;

mod context;
mod deptable;
mod discover;
mod set_up_runner;
mod set_up_workers;
mod tasklist;
mod tear_down_runner;

mod progress;


use crate::discover::discover_setups;
use crate::progress::{OverallResult, OverallSummaryBuilder, Phase, PhaseResult, PhaseSummary, PhaseSummaryBuilder, ProgressMonitor, TaskStatus};
use crate::set_up_runner::run_set_ups;
use crate::tear_down_runner::run_tear_downs;
pub use context::{Context, GlobalContext, Param};
use libtest_mimic::{Arguments, Trial};

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

pub type SetFnOutput = Pin<Box<dyn Future<Output = SetUpResult> + Send + 'static>>;

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

fn run_tests() -> PhaseSummary {
    let args = Arguments::from_args();
    let mut tests = Vec::new();

    let mut bld = PhaseSummaryBuilder::new(Phase::Test);

    for test in inventory::iter::<RegisteredITest> {
        tests.push(Trial::test(test.name.to_owned(), move || {
            (test.test_fn)();
            Ok(())
        }));
    }

    let conclusion = libtest_mimic::run(&args, tests);

    bld.add( conclusion.num_passed as usize, TaskStatus::Ok);
    bld.add( conclusion.num_ignored as usize, TaskStatus::Skipped);
    bld.add( conclusion.num_failed  as usize, TaskStatus::Failed);

    bld.build()
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
        let result = rt.block_on(self.run_async());
        if result == OverallResult::Ok {
            ExitCode::SUCCESS.exit_process()
        } else {
            ExitCode::FAILURE.exit_process()
        }
    }

    async fn run_async(self) -> OverallResult {
        let mut summary = OverallSummaryBuilder::new();
        let workspace_root_dir = find_workspace_root_dir();

        let set_ups = discover_setups().unwrap();
        let task_names = set_ups.tasks().map(|(t, n)| (t, n.to_string())).collect();

        let monitor = ProgressMonitor::new(task_names);

        let mut global_ctx = GlobalContext::new(&workspace_root_dir);

        let (tear_downs, set_up_outcome) =
            run_set_ups(set_ups, &mut global_ctx, monitor.listener()).await;

        summary.add_phase(set_up_outcome.clone());

        let test_outcome = if set_up_outcome.result == PhaseResult::Ok {
            run_tests()
        } else {
           PhaseSummary::skipped(Phase::Test)
        };
        summary.add_phase(test_outcome);

        let tear_down_outcome = run_tear_downs(monitor.listener(), tear_downs).await;
        summary.add_phase(tear_down_outcome);

        let summary = summary.build();

        monitor.listener().finished(summary.clone()).await;

        monitor.shutdown().await;

        summary.result
    }
}
