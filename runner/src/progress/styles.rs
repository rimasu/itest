use anstyle::{AnsiColor, Color, Style};
use crate::progress::{OverallResult, PhaseResult, TaskStatus};

pub struct Styles {
    pub bold: Style,
    pub bad: Style,
    good: Style,
    norm: Style,
}

impl Styles {
    pub fn task_status(&self, status: TaskStatus) -> Style {
        match status {
            TaskStatus::Running => self.norm,
            TaskStatus::Failed => self.bad,
            TaskStatus::Ok => self.good,
            TaskStatus::Skipped => self.norm,
        }
    }

    pub fn phase_result(&self, result: PhaseResult) -> Style {
        match result {
            PhaseResult::Ok => self.good,
            PhaseResult::Failed => self.bad,
            PhaseResult::Skipped => self.norm,
        }
    }

    pub fn overall_result(&self, result: OverallResult) -> Style {
        match result {
            OverallResult::Ok => self.good,
            OverallResult::Failed => self.bad,
        }
    }
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            bold: Style::new().bold(),
            bad: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightRed))),
            good: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightGreen))),
            norm: Style::new().fg_color(Some(Color::Ansi(AnsiColor::White))),
        }
    }
}
