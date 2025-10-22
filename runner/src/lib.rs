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
mod phases;
mod tasklist;

mod progress;


use crate::discover::{discover_setups, discover_tests, SetUps, Tests};
use crate::progress::{OverallResult, ProgressListener, ProgressMonitor};

use tasklist::Task;
pub use context::{Context, GlobalContext, Param};

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

pub type TestFn = fn();

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
    pub test_fn: TestFn,
    pub file: &'static str,
    pub line: usize,
}
inventory::collect!(RegisteredITest);

#[async_trait]
pub trait TearDown: Send {
    async fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Default)]
pub struct TearDowns {
    tear_downs: Vec<(Task, Box<dyn TearDown + 'static>)>
}

impl TearDowns {
    pub fn push(&mut self, task: Task, tear_down: Box<dyn TearDown + 'static>) {
        self.tear_downs.push((task, tear_down))
    }

    pub fn len(&self) -> usize {
        self.tear_downs.len()
    }

    pub fn pop(&mut self) -> Option<(Task,Box<dyn TearDown + 'static>)> {
        self.tear_downs.pop()
    }
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
        let set_ups = discover_setups().unwrap();
        let tests = discover_tests().unwrap();
        let task_names = set_ups.tasks().map(|(t, n)| (t, n.to_string())).collect();
       
        let monitor = ProgressMonitor::new(task_names);
        let progress = monitor.listener();
        let result = self.run_with_monitor(set_ups, tests, &progress).await;
        monitor.shutdown().await;

        result
    }

    async fn run_with_monitor(self, 
        set_ups: SetUps,
        tests: Tests,
        progress: &ProgressListener,
    ) -> OverallResult {
   
        let workspace_root_dir = find_workspace_root_dir();
     
        let mut global_ctx = GlobalContext::new(&workspace_root_dir);

        phases::run(&mut global_ctx,  set_ups, tests, progress).await
    }
}
