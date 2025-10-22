use std::time::Instant;

use async_channel::Receiver;
use tokio::sync::mpsc;

use crate::{
    discover::SetUps, progress::{Phase, PhaseSummary, PhaseSummaryBuilder, ProgressListener, TaskStatus}, tasklist::{Status, Task}, Context, GlobalContext, SetUpError, SetUpFn, SetUpResult, TearDown, TearDowns
};

pub async fn run(
    set_ups: SetUps,
    global_ctx: &mut GlobalContext,
    progress: &ProgressListener,
) -> (TearDowns, PhaseSummary) {
    let mut workers = launch_set_up_workers(3, progress.clone());

    let mut tear_downs = TearDowns::default();
    let mut errs: Vec<SetUpError> = Vec::new();

    progress
        .phase_started(Phase::SetUp, set_ups.tasks().count())
        .await;

    let mut tasks = set_ups.make_task_list();

    // push the task that are ready to go
    if let Some(ready) = tasks.pop_ready() {
        for task in ready {
            let ctx = global_ctx.create_component_context(set_ups.dep_table.name(task.0));
            let set_up = set_ups.dep_table.decl(task.0).set_up_fn;
            workers.push(task, set_up, ctx).await;
        }
    }

    let mut summary = PhaseSummaryBuilder::new(Phase::SetUp);

    while let Some((task, result)) = workers.pull_result().await {
        match result {
            Ok(out) => {
                if let Some(tear_down) = out {
                    tear_downs.push(task, tear_down);
                }
                tasks.set_status(task, Status::Success);
                summary.inc(TaskStatus::Ok);
            }
            Err(e) => {
                tasks.set_status(task, Status::Failed);
                errs.push(e);
                summary.inc(TaskStatus::Failed);
            }
        }

        if let Some(ready) = tasks.pop_ready() {
            for task in ready {
                let ctx = global_ctx.create_component_context(set_ups.dep_table.name(task.0));
                let set_up = set_ups.dep_table.decl(task.0).set_up_fn;
                workers.push(task, set_up, ctx).await;
            }
        }

        if tasks.none_waiting() {
            break;
        }
    }

    let summary = summary.build();
    progress.phase_finished(summary.clone()).await;

    (tear_downs, summary)
}

fn launch_set_up_workers(num_workers: usize, progress: ProgressListener) -> SetUpWorkers {
    let (run_tx, run_rx) = async_channel::unbounded();
    let (result_tx, result_rx) = mpsc::channel(100);
    for _ in 1..=num_workers {
        let run_rx: Receiver<(Task, &'static SetUpFn, Context)> = run_rx.clone();
        let result_tx = result_tx.clone();
        let progress = progress.clone();
        tokio::spawn(async move {
            while let Ok((task, set_up, ctx)) = run_rx.recv().await {
                progress.task_running(Phase::SetUp, task).await;
                let start = Instant::now();
                let r = (*set_up)(ctx).await;
                let duration = start.elapsed();
                match &r {
                    Ok(_) => {
                        progress.task_done(Phase::SetUp, task, duration).await;
                    }
                    Err(err) => {
                        progress
                            .task_failed(Phase::SetUp, task, duration, format!("{:?}", err))
                            .await;
                    }
                }

                if let Some(err) = result_tx.send((task, r)).await.err() {
                    eprintln!("Failed to publish result of task {task:?} {err:?}");
                }
            }
        });
    }

    SetUpWorkers { run_tx, result_rx }
}

struct SetUpWorkers {
    run_tx: async_channel::Sender<(Task, &'static SetUpFn, Context)>,
    result_rx: mpsc::Receiver<(Task, SetUpResult)>,
}

impl SetUpWorkers {
    pub async fn push(&self, task: Task, set_up_fn: &'static SetUpFn, ctx: Context) {
        self.run_tx.send((task, set_up_fn, ctx)).await.unwrap();
    }

    pub async fn pull_result(&mut self) -> Option<(Task, SetUpResult)> {
        self.result_rx.recv().await
    }
}
