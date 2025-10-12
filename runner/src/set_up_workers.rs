use std::time::Instant;

use async_channel::Receiver;
use tokio::sync::mpsc;

use crate::{
    Context, SetUpFn, SetUpResult,
    progress::{Phase, ProgressListener},
    tasklist::Task,
};

pub fn launch_set_up_workers(num_workers: usize, progress: ProgressListener) -> SetUpWorkers {
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

    SetUpWorkers {
        run_tx,
        result_rx,
        progress,
    }
}

pub struct SetUpWorkers {
    run_tx: async_channel::Sender<(Task, &'static SetUpFn, Context)>,
    result_rx: mpsc::Receiver<(Task, SetUpResult)>,
    progress: ProgressListener,
}

impl SetUpWorkers {
    pub async fn push(&self, task: Task, set_up_fn: &'static SetUpFn, ctx: Context) {
        self.progress.task_ready(Phase::SetUp, task).await;
        self.run_tx.send((task, set_up_fn, ctx)).await.unwrap();
    }

    pub async fn pull_result(&mut self) -> Option<(Task, SetUpResult)> {
        self.result_rx.recv().await
    }
}
