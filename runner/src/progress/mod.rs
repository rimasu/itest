use std::fmt;

mod styles;
mod status;
mod monitor;
mod summary;


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Phase {
    SetUp,
    Test,
    TearDown,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Phase::SetUp => "set up",
            Phase::Test => "test",
            Phase::TearDown => "tear down",
        };
        fmt::Display::fmt(s, f)
    }
}


pub use status::{TaskStatus, PhaseResult, OverallResult};
pub use monitor::{ProgressListener, ProgressMonitor};
pub use summary::{OverallSummary, OverallSummaryBuilder, PhaseSummary, PhaseSummaryBuilder};


