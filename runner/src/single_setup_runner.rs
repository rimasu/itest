use std::{collections::HashMap, time::Instant};

use crate::{
    GlobalContext, TearDown,
    discover::SetUps,
    progress::ProgressListener,
    tasklist::{Status, Task},
};

pub struct SetUpOutcome {
    pub success: bool,
    pub tear_downs: Vec<(String, Box<dyn TearDown + 'static>)>,
}

pub async fn run_set_ups(
    set_ups: SetUps,
    ctx: &mut GlobalContext,
    progress: ProgressListener,
) -> SetUpOutcome {
    let mut tear_downs = Vec::new();
    let mut errs = Vec::new();

    // let mut status_table = StatusTable::new(&set_ups);

    let start = Instant::now();
    progress.set_ups_started().await;

    let mut tasks = set_ups.make_task_list();
    while let Some(ready) = tasks.pop_ready() {
        for task in &ready {
            progress.set_up_ready(*task).await;
        }

        for task in ready {
            let context2 = ctx.create_component_context(set_ups.dep_table.name(task.0));
            let set_up = set_ups.dep_table.decl(task.0).set_up_fn;
            tasks.set_status(task, Status::Running);

            progress.set_up_started(task).await;

            let set_up_start = Instant::now();
            let r = (*set_up)(context2).await;
            let set_up_duration = set_up_start.elapsed();

            match r {
                Ok(output) => {
                    progress.set_up_finished(task, set_up_duration).await;
                    tasks.set_status(task, Status::Finished);
                    if let Some(tear_down) = output {
                        tear_downs.push((set_ups.dep_table.name(task.0).to_owned(), tear_down));
                    }
                }
                Err(err) => {
                    progress
                        .set_up_failed(task, set_up_duration, &format!("{:?}", err))
                        .await;
                    tasks.set_status(task, Status::Failed);
                    errs.push((set_ups.dep_table.name(task.0), format!("{:?}", err)));
                }
            }

            if !errs.is_empty() {
                break;
            }
        }
    }

    let success = tasks.all_finished();
    let set_up_duration = start.elapsed();
    progress.set_ups_finished(success, set_up_duration).await;

    for (name, err) in errs {
        println!("{} failed\n\t{}", name, err);
    }

    SetUpOutcome {
        success: tasks.all_finished(),
        tear_downs,
    }
}
