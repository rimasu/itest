use std::time::Instant;

use crate::{
    GlobalContext, SetUpError, TearDown,
    discover::SetUps,
    progress::{Phase, ProgressListener},
    set_up_workers::launch_set_up_workers,
    tasklist::{Status, Task},
};

pub struct SetUpOutcome {
    pub success: bool,
    pub tear_downs: Vec<(Task, Box<dyn TearDown + 'static>)>,
}

pub async fn run_set_ups(
    set_ups: SetUps,
    global_ctx: &mut GlobalContext,
    progress: ProgressListener,
) -> SetUpOutcome {
    let mut workers = launch_set_up_workers(3, progress.clone());

    let mut tear_downs = Vec::new();
    let mut errs: Vec<SetUpError> = Vec::new();

    let phase_start = Instant::now();
    progress.phase_started(Phase::SetUp).await;

    let mut tasks = set_ups.make_task_list();

    // push the task that are ready to go
    if let Some(ready) = tasks.pop_ready() {
        for task in ready {
            let ctx = global_ctx.create_component_context(set_ups.dep_table.name(task.0));
            let set_up = set_ups.dep_table.decl(task.0).set_up_fn;
            workers.push(task, set_up, ctx).await;
        }
    }

    while let Some((task, result)) = workers.pull_result().await {
        match result {
            Ok(out) => {
                if let Some(tear_down) = out {
                    tear_downs.push((task, tear_down));
                }
                tasks.set_status(task, Status::Success);
            }
            Err(e) => {
                tasks.set_status(task, Status::Failed);
                errs.push(e);
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
    let phase_duration = phase_start.elapsed();
    progress.phase_finished(Phase::SetUp, phase_duration).await;

    SetUpOutcome {
        success,
        tear_downs,
    }
}
