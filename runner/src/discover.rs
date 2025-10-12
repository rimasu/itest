use std::fmt;

use crate::{
    RegisteredSetUp, SetUpFn,
    deptable::{Builder, DepTable},
    tasklist::{Status, Task, TaskList},
};

pub struct SetUps {
    display_order: Vec<Task>,
    pub dep_table: DepTable<SetUpDecl>,
}

impl SetUps {
    pub fn make_task_list(&self) -> TaskList {
        self.dep_table.make_task_list()
    }

    pub fn tasks(&self) -> impl Iterator<Item = (Task, &str)> {
        self.display_order
            .iter()
            .map(|task| (*task, self.dep_table.name(task.0)))
    }
}

pub struct SetUpDecl {
    pub set_up_fn: &'static SetUpFn,
    file: String,
    line: usize,
}

impl fmt::Display for SetUpDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.file, self.line)
    }
}

fn build_dep_table() -> Result<DepTable<SetUpDecl>, ()> {
    let mut bld = Builder::new();

    for set_up in inventory::iter::<RegisteredSetUp> {
        let decl = SetUpDecl {
            set_up_fn: &set_up.set_up_fn,
            file: set_up.file.to_owned(),
            line: set_up.line,
        };

        bld.declare_node(decl, set_up.name, set_up.deps);
    }

    match bld.build() {
        Ok(dep_table) => Ok(dep_table),
        Err(errs) => {
            for err in errs {
                eprintln!("{}", err);
            }
            Err(())
        }
    }
}

fn dry_run_tasks(dep_table: &DepTable<SetUpDecl>) -> Result<Vec<Task>, ()> {
    let mut task = dep_table.make_task_list();
    let mut dry_run_order = Vec::new();

    while let Some(mut ready) = task.pop_ready() {
        // sort all the tasks in the ready list by their name
        ready.sort_by(|a, b| dep_table.name(a.0).cmp(dep_table.name(b.0)));
        dry_run_order.extend_from_slice(&ready);

        // mark them all as complete
        for idx in ready {
            task.set_status(idx, Status::Success);
        }
    }

    if task.all_success() {
        Ok(dry_run_order)
    } else {
        // could not find valid order
        Err(())
    }
}

pub fn discover_setups() -> Result<SetUps, ()> {
    let dep_table = build_dep_table()?;
    let display_order = dry_run_tasks(&dep_table)?;
    Ok(SetUps {
        display_order,
        dep_table,
    })
}
