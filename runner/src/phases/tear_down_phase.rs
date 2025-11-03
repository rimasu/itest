use futures::FutureExt;
use std::{
    any::Any,
    panic::{ AssertUnwindSafe},
    time::{Instant},
};

use crate::{
    TearDown, TearDownResult, TearDowns, progress::{Phase, PhaseSummary, PhaseSummaryBuilder, ProgressListener, TaskStatus}, tasklist::Task
};

pub async fn run(progress: &ProgressListener, mut tear_downs: TearDowns) -> PhaseSummary {

    let mut summary = PhaseSummaryBuilder::new(Phase::TearDown);

    progress
        .phase_started(Phase::TearDown, tear_downs.len())
        .await;

    while let Some((task, tear_down)) = tear_downs.pop() {
        run_task(task, &mut summary, tear_down, progress).await;
    }

    let summary = summary.build();
    progress.phase_finished(summary.clone()).await;

    summary
}

async fn run_task(
    task: Task,
    summary: &mut PhaseSummaryBuilder,
    tear_down: Box<dyn TearDown + 'static>,
    progress: &ProgressListener,
)  {

    progress.task_running(Phase::TearDown, task).await;

    let start = Instant::now();
    let result = safe_run_task(tear_down).await;
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
}


fn panic_err(e: Box<dyn Any + Send>) -> Box<dyn  std::error::Error> {
    format!("Setup panicked during execution: {:?}", e).into()
}


async fn safe_run_task(mut tear_down: Box<dyn TearDown + 'static>) -> TearDownResult {
    match AssertUnwindSafe(tear_down.tear_down()).catch_unwind().await {
        Ok(result) => result,
        Err(panic) => Err(panic_err(panic)),
    }
}