use std::fmt;

mod monitor;
mod summary;

pub use monitor::{ProgressListener, ProgressMonitor};
pub use summary::{Summary, SummaryBuilder, PhaseSummary, PhaseSummaryBuilder};


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Phase {
    SetUp,
    TearDown,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Phase::SetUp => "set up",
            Phase::TearDown => "tear down",
        };
        fmt::Display::fmt(s, f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TaskStatus {
    Running,
    Failed,
    Ok,
    Skipped,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TaskStatus::Running => "running",
            TaskStatus::Failed => "failed",
            TaskStatus::Ok => "ok",
            TaskStatus::Skipped => "skipped",
        };
        fmt::Display::fmt(s, f)
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PhaseResult {
    Failed,
    Ok,
    Skipped,
}

impl fmt::Display for PhaseResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            PhaseResult::Failed => "failed",
            PhaseResult::Ok => "ok",
            PhaseResult::Skipped => "skipped",
        };
        fmt::Display::fmt(s, f)
    }
}
