use std::fmt;
use std::time::Instant;

use crate::{
    TearDown,
    progress::{Phase, PhaseSummary, PhaseSummaryBuilder, ProgressListener, TaskStatus},
    tasklist::Task,
};

impl fmt::Debug for TearDownOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "     ok: {}", self.num_ok)?;
        writeln!(f, " failed: {}", self.num_failed)?;
        Ok(())
    }
}

pub struct TearDownOutcome {
    num_ok: usize,
    num_failed: usize,
}

pub async fn run_tear_downs(
    progress: ProgressListener,
    tear_downs: Vec<(Task, Box<dyn TearDown + 'static>)>,
) -> PhaseSummary {
    let mut tear_down_result = Vec::new();

    let mut summary = PhaseSummaryBuilder::new();

    progress.phase_started(Phase::TearDown, tear_downs.len()).await;
    for (task, mut tear_down) in tear_downs.into_iter().rev() {
        progress.task_running(Phase::TearDown, task).await;

        let start = Instant::now();
        let result = (*tear_down).tear_down().await;
        let duration = start.elapsed();

        match &result {
            Ok(()) => {
                summary.inc(TaskStatus::Ok);
                progress.task_done(Phase::TearDown, task, duration).await;
            }
            Err(e) => {
                summary.inc(TaskStatus::Failed);
                progress
                    .task_failed(Phase::TearDown, task, duration, format!("{:?}", e))
                    .await
            }
        }
        tear_down_result.push(result);
    }

    let summary = summary.build();
    progress
        .phase_finished(Phase::TearDown, summary.clone())
        .await;

    summary
}
