use std::{collections::HashMap, fmt};

use indicatif::{MultiProgress, MultiProgressAlignment, ProgressBar, ProgressStyle, style};

use crate::{
    Context, GlobalContext, RegisteredSetUp, SetUpFn, TearDown,
    deptable::{Builder, DepTable},
    tasklist::Status,
};

struct SetUpDecl {
    set_up_fn: &'static SetUpFn,
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

fn max_name_len(tasks: &[usize], dep_table: &DepTable<SetUpDecl>) -> usize {
    tasks
        .iter()
        .map(|d| dep_table.name(*d).len())
        .max()
        .unwrap_or(1)
}

pub async fn run_set_ups(ctx: &mut GlobalContext) -> Result<Vec<(String, Box<dyn TearDown>)>, ()> {
    let dep_table = build_dep_table()?;

    let mut tear_downs = Vec::new();
    let order = dry_run_tasks(&dep_table)?;
    let m = MultiProgress::new();

    let max_name_len = max_name_len(&order, &dep_table);

    println!("Running setups\n");

    m.set_alignment(MultiProgressAlignment::Top);
    let mut spinners = HashMap::new();
    for idx in &order {
        let name = dep_table.name(*idx).to_string();
        let item = m.add(ProgressBar::new_spinner());
        item.enable_steady_tick(std::time::Duration::from_millis(100));
        item.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix:.bold} {msg}")
                .unwrap(),
        );
        item.set_prefix(format!("{:>width$}:", name, width = max_name_len));
        item.set_message("waiting");

        spinners.insert(*idx, item);
    }
    let mut task = dep_table.make_task_list();
    while let Some(ready) = task.pop_all_ready() {
        for idx in ready {
            let context2 = ctx.create_component_context(dep_table.name(idx));
            let set_up = dep_table.decl(idx).set_up_fn;
            task.set_status(idx, Status::Running);
            spinners.get(&idx).unwrap().set_message("running");
            let r = run_set_up(context2, set_up).await;
            spinners
                .get(&idx)
                .unwrap()
                .finish_with_message(format!("ok"));
            task.set_status(idx, Status::Finished);
            if let Ok(Some(tear_down)) = r {
                tear_downs.push((dep_table.name(idx).to_owned(), tear_down));
            }
        }
    }

    println!("\n");
    println!("Setup Complete");

    Ok(tear_downs)
}

async fn run_set_up(
    ctx: Context,
    set_up: &SetUpFn,
) -> Result<Option<Box<dyn TearDown>>, Box<dyn std::error::Error>> {
    (*set_up)(ctx).await
}
