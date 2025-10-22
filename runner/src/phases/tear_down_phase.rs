
use std::time::Instant;

use crate::{
    progress::{Phase, PhaseSummary, PhaseSummaryBuilder, ProgressListener, TaskStatus}, TearDowns
};

pub async fn run(
    progress: &ProgressListener,
    mut tear_downs: TearDowns,
) -> PhaseSummary {
    let mut tear_down_result = Vec::new();

    let mut summary = PhaseSummaryBuilder::new(Phase::TearDown);

    progress
        .phase_started(Phase::TearDown, tear_downs.len())
        .await;

    while let Some((task, mut tear_down)) =  tear_downs.pop() {
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
    progress.phase_finished(summary.clone()).await;

    summary
}
