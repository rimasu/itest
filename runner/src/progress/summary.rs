use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

use crate::progress::{OverallResult, Phase, PhaseResult, TaskStatus};

pub struct OverallSummaryBuilder {
    start: Instant,
    phases: Vec<PhaseSummary>,
}

impl OverallSummaryBuilder {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            phases: Vec::new(),
        }
    }

    pub fn add_phase(&mut self, summary: PhaseSummary) {
        self.phases.push(summary);
    }

    fn result(&self) -> OverallResult {
        let all_phases_ok = self.phases.iter().all(|p| p.result == PhaseResult::Ok);
        if all_phases_ok {
            OverallResult::Ok
        } else {
            OverallResult::Failed
        }
    }

    pub fn build(self) -> OverallSummary {
        let result = self.result();
        OverallSummary {
            result,
            duration: self.start.elapsed(),
            phases: self.phases,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OverallSummary {
    pub result: OverallResult,
    pub duration: Duration,
    pub phases: Vec<PhaseSummary>,
}

pub struct PhaseSummaryBuilder {
    phase: Phase,
    start: Instant,
    counts: HashMap<TaskStatus, usize>,
}

impl PhaseSummaryBuilder {
    pub fn new(phase: Phase) -> Self {
        Self {
            phase,
            start: Instant::now(),
            counts: HashMap::new(),
        }
    }

    pub fn inc(&mut self, status: TaskStatus) {
        *(self.counts.entry(status).or_default()) += 1;
    }

    fn all_tasks_ok(&self) -> bool {
        let total: usize = self.counts.values().sum();
        let okay = *self.counts.get(&TaskStatus::Ok).unwrap_or(&0);
        okay == total
    }

    fn result(&self) -> PhaseResult {
        if self.all_tasks_ok() {
            PhaseResult::Ok
        } else {
            PhaseResult::Failed
        }
    }

    pub fn build(self) -> PhaseSummary {
        let result = self.result();
        let duration = self.start.elapsed();
        PhaseSummary {
            phase: self.phase,
            result,
            duration,
            counts: self.counts,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PhaseSummary {
    pub phase: Phase,
    pub result: PhaseResult,
    pub duration: Duration,
    pub counts: HashMap<TaskStatus, usize>,
}
