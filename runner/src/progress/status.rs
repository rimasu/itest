use std::fmt;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum OverallResult {
    Failed,
    Ok
}

impl fmt::Display for OverallResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            OverallResult::Failed => "failed",
            OverallResult::Ok => "ok",
        };
        fmt::Display::fmt(s, f)
    }
}