use std::collections::HashMap;

use crate::{
    GlobalContext, TearDown,
    discover::SetUps,
    tasklist::{Status, Task},
};

use indicatif::{MultiProgress, MultiProgressAlignment, ProgressBar, ProgressStyle};

pub struct SetUpOutcome {
    pub success: bool,
    pub tear_downs: Vec<(String, Box<dyn TearDown + 'static>)>,
}

struct StatusTable {
    m: MultiProgress,
    spinners: HashMap<Task, ProgressBar>,
}

impl StatusTable {
    pub fn new(set_ups: &SetUps) -> StatusTable {
        let m = MultiProgress::new();
        let max_name_len = set_ups.max_name_len();
        m.set_alignment(MultiProgressAlignment::Top);
        let mut spinners = HashMap::new();
        for (idx, name) in set_ups.tasks() {
            let item = m.add(ProgressBar::new_spinner());
            item.enable_steady_tick(std::time::Duration::from_millis(100));
            item.set_style(
                ProgressStyle::default_spinner()
                    .template("{prefix:.bold} {msg}")
                    .unwrap(),
            );
            item.set_prefix(format!("{:>width$}:", name, width = max_name_len));
            item.set_message("waiting");

            spinners.insert(idx, item);
        }
        StatusTable { m, spinners }
    }

    pub fn set_status(&mut self, task: Task, status: Status) {
        match status {
            Status::Waiting => todo!(),
            Status::Ready => todo!(),
            Status::Running => self.spinners.get(&task).unwrap().set_message("running"),
            Status::Finished => self
                .spinners
                .get(&task)
                .unwrap()
                .finish_with_message("finished"),
            Status::Skipped => self
                .spinners
                .get(&task)
                .unwrap()
                .finish_with_message("skipped"),
            Status::Failed => self
                .spinners
                .get(&task)
                .unwrap()
                .finish_with_message("skipped"),
        }
    }
}

pub async fn run_set_ups(set_ups: SetUps, ctx: &mut GlobalContext) -> SetUpOutcome {
    let mut tear_downs = Vec::new();
    let mut errs = Vec::new();

    let mut status_table = StatusTable::new(&set_ups);

    println!("Running setups\n");

    let mut tasks = set_ups.make_task_list();
    while let Some(ready) = tasks.pop_ready() {
        for task in ready {
            let context2 = ctx.create_component_context(set_ups.dep_table.name(task.0));
            let set_up = set_ups.dep_table.decl(task.0).set_up_fn;
            tasks.set_status(task, Status::Running);
            status_table.set_status(task, Status::Running);

            let r = (*set_up)(context2).await;

            match r {
                Ok(output) => {
                    status_table.set_status(task, Status::Finished);
                    tasks.set_status(task, Status::Finished);
                    if let Some(tear_down) = output {
                        tear_downs.push((set_ups.dep_table.name(task.0).to_owned(), tear_down));
                    }
                }
                Err(err) => {
                    status_table.set_status(task, Status::Failed);
                    tasks.set_status(task, Status::Failed);
                    errs.push((set_ups.dep_table.name(task.0), format!("{:?}", err)));
                }
            }

            if !errs.is_empty() {
                break;
            }
        }
    }

    println!("\n");
    println!("Setup Complete");

    for (name, err) in errs {
        println!("{} failed\n\t{}", name, err);
    }

    SetUpOutcome {
        success: tasks.all_finished(),
        tear_downs,
    }
}
