use std::fmt;

use crate::{
    Context, RegisteredSetUp, SetUpFunc,
    deptable::{Builder, DepTable, Error},
    tasklist::Status,
};

struct SetUpDecl {
    set_up_fn: &'static SetUpFunc,
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

fn dry_run_tasks(dep_table: &DepTable<SetUpDecl>) -> Result<Vec<usize>, ()> {
    let mut task = dep_table.make_task_list();
    let mut dry_run_order = Vec::new();

    while let Some(mut ready) = task.pop_all_ready() {
        // sort all the tasks in the ready list by their name
        ready.sort_by(|a, b| dep_table.name(*a).cmp(dep_table.name(*b)));
        dry_run_order.extend_from_slice(&ready);

        // mark them all as complete
        for idx in ready {
            task.set_status(idx, Status::Running);
            task.set_status(idx, Status::Finished);
        }
    }

    Ok(dry_run_order)
}

pub async fn run_set_ups(ctx: &mut Context) -> Result<(), ()> {
    let dep_table = build_dep_table()?;

    let order = dry_run_tasks(&dep_table)?;
    println!("Report order: {:?}", order);

    let mut task = dep_table.make_task_list();
    while let Some(ready) = task.pop_all_ready() {
        for idx in ready {
            ctx.set_current_component(dep_table.name(idx));
            let set_up = dep_table.decl(idx).set_up_fn;
            task.set_status(idx, Status::Running);
            let r = run_set_up(ctx, set_up).await;
            println!("{:?}", r);
            task.set_status(idx, Status::Finished);
        }
    }

    Ok(())
}

async fn run_set_up(
    ctx: &mut Context,
    set_up: &SetUpFunc,
) -> Result<(), Box<dyn std::error::Error>> {
    match set_up {
        SetUpFunc::Sync(set_up) => (*set_up)(ctx),
        SetUpFunc::Async(set_up) => (*set_up)(ctx).await,
    }
}
