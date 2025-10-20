
use crate::{
    GlobalContext, SetUpError, TearDown,
    discover::SetUps,
    progress::{Phase, PhaseSummary, PhaseSummaryBuilder, ProgressListener, TaskStatus},
    set_up_workers::launch_set_up_workers,
    tasklist::{Status, Task},
};


pub async fn run_set_ups(
    set_ups: SetUps,
    global_ctx: &mut GlobalContext,
    progress: ProgressListener,
) -> (Vec<(Task, Box<dyn TearDown + 'static>)>, PhaseSummary) {
    let mut workers = launch_set_up_workers(3, progress.clone());

    let mut tear_downs = Vec::new();
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
                    tear_downs.push((task, tear_down));
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
