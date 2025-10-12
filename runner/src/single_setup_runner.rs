use std::time::Instant;

use async_channel::Receiver;
use tokio::sync::mpsc;

use crate::{
    Context, GlobalContext, SetUpFn, SetUpResult, TearDown,
    discover::SetUps,
    progress::ProgressListener,
    tasklist::{Status, Task},
};

pub struct SetUpOutcome {
    pub success: bool,
    pub tear_downs: Vec<(Task, Box<dyn TearDown + 'static>)>,
}

pub struct Workers {
    run_tx: async_channel::Sender<(Task, &'static SetUpFn, Context)>,
    result_rx: mpsc::Receiver<(Task, SetUpResult)>,
    progress: ProgressListener,
}

impl Workers {
    pub async fn push(&self, task: Task, set_up_fn: &'static SetUpFn, ctx: Context) {
        self.progress.set_up_ready(task).await;
        self.run_tx.send((task, set_up_fn, ctx)).await.unwrap();
    }
}

pub fn launch_set_up_workers(num_workers: usize, progress: ProgressListener) -> Workers {
    let (run_tx, run_rx) = async_channel::unbounded();
    let (result_tx, result_rx) = mpsc::channel(100);
    for _ in 1..=num_workers {
        let run_rx: Receiver<(Task, &'static SetUpFn, Context)> = run_rx.clone();
        let result_tx = result_tx.clone();
        let progress = progress.clone();
        tokio::spawn(async move {
            while let Ok((task, set_up, ctx)) = run_rx.recv().await {
                progress.set_up_started(task).await;
                let start = Instant::now();
                let r = (*set_up)(ctx).await;
                let duration = start.elapsed();
                match &r {
                    Ok(_) => {
                        progress.set_up_finished(task, duration).await;
                    }
                    Err(err) => {
                        progress
                            .set_up_failed(task, duration, &format!("{:?}", err))
                            .await;
                    }
                }

                if let Some(err) = result_tx.send((task, r)).await.err() {
                    eprintln!("Failed to publish result of task {task:?} {err:?}");
                }
            }
        });
    }

    Workers {
        run_tx,
        result_rx,
        progress,
    }
}

pub async fn run_set_ups(
    set_ups: SetUps,
    global_ctx: &mut GlobalContext,
    progress: ProgressListener,
) -> SetUpOutcome {
    let mut workers = launch_set_up_workers(3, progress.clone());

    let mut tear_downs = Vec::new();
    let mut errs: Vec<Box<dyn std::error::Error>> = Vec::new();

    let start = Instant::now();
    progress.set_ups_started().await;

    let mut tasks = set_ups.make_task_list();

    // push the task that are ready to go
    if let Some(ready) = tasks.pop_ready() {
        for task in ready {
            let ctx = global_ctx.create_component_context(set_ups.dep_table.name(task.0));
            let set_up = set_ups.dep_table.decl(task.0).set_up_fn;
            workers.push(task, set_up, ctx).await;
        }
    }

    while let Some((task, result)) = workers.result_rx.recv().await {
        match result {
            Ok(out) => {
                if let Some(tear_down) = out {
                    tear_downs.push((task, tear_down));
                }
                tasks.set_status(task, Status::Success);
            }
            Err(e) => {
                tasks.set_status(task, Status::Failed);
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

    let success = tasks.all_success();
    let set_up_duration = start.elapsed();
    progress.set_ups_finished(success, set_up_duration).await;

    SetUpOutcome {
        success: tasks.all_success(),
        tear_downs,
    }
}
