#![feature(exit_status_error)]

use std::pin::Pin;
use std::{fmt, path::PathBuf, process::Command};

use async_trait::async_trait;
pub use inventory::{collect, submit};
pub use itest_macros::{depends_on, itest, set_up};

pub mod components;

mod context;
mod deptable;
mod discover;
mod tasklist;

pub use context::{Context, GlobalContext, Param};

use libtest_mimic::{Arguments, Conclusion, Trial};

use crate::discover::run_set_ups;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Outcome {
    Ok,
    Failed,
    Skipped,
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Outcome::Skipped => "skipped",
            Outcome::Ok => "ok",
            Outcome::Failed => "FAILED",
        })
    }
}

type SyncSetUpFn = fn(Context) -> Result<(), Box<dyn std::error::Error>>;

type AsyncSetUpFn = fn(
    Context,
) -> Pin<
    Box<dyn Future<Output = Result<Option<Box<dyn TearDown>>, Box<dyn std::error::Error>>>>,
>;

pub enum SetUpFunc {
    Sync(SyncSetUpFn),
    Async(AsyncSetUpFn),
}
inventory::collect!(RegisteredSetUp);

pub struct RegisteredSetUp {
    pub name: &'static str,
    pub set_up_fn: SetUpFunc,
    pub deps: &'static [&'static str],
    pub file: &'static str,
    pub line: usize,
}

pub struct RegisteredITest {
    pub name: &'static str,
    pub test_fn: fn(),
}
inventory::collect!(RegisteredITest);

// pub type SetUpResult = Result<Box<dyn TearDown + 'static>, Box<dyn std::error::Error>>;

#[async_trait]
pub trait TearDown {
    async fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

// struct Component {
//     name: String,
//     set_up_fn: SetupFunction,
//     set_up_err: Option<Box<dyn std::error::Error>>,
//     tear_down: Option<Box<dyn TearDown + 'static>>,
//     tear_down_err: Option<Box<dyn std::error::Error>>,
// }

// impl Component {
//     fn new(name: &str, set_up_fn: SetupFunction) -> Component {
//         Component {
//             name: name.to_owned(),
//             set_up_fn,
//             set_up_err: None,
//             tear_down: None,
//             tear_down_err: None,
//         }
//     }

//     async fn set_up(&mut self, ctx: &mut Context) -> Outcome {
//         ctx.set_current_component(&self.name);

//         let outcome = match (self.set_up_fn)(ctx) {
//             Ok(mut set_up) => match set_up.set_up(ctx).await {
//                 Ok(tear_down) => {
//                     self.tear_down = Some(tear_down);
//                     Outcome::Ok
//                 }
//                 Err(err) => {
//                     self.set_up_err = Some(err);
//                     Outcome::Failed
//                 }
//             },
//             Err(err) => {
//                 self.set_up_err = Some(err);
//                 Outcome::Failed
//             }
//         };

//         outcome
//     }

//     async fn tear_down(&mut self) -> Outcome {
//         let outcome = if let Some(tear_down) = &mut self.tear_down {
//             match tear_down.tear_down().await {
//                 Ok(()) => Outcome::Ok,
//                 Err(err) => {
//                     self.tear_down_err = Some(err);
//                     Outcome::Failed
//                 }
//             }
//         } else {
//             Outcome::Skipped
//         };
//         outcome
//     }
// }

// #[derive(Default)]
// struct Components {
//     components: Vec<Component>,
// }

// impl Components {
//     fn add_component(&mut self, name: &str, set_up_fn: SetupFunction) {
//         self.components.push(Component::new(name, set_up_fn));
//     }

//     pub(crate) fn max_component_name_len(&self) -> usize {
//         self.components
//             .iter()
//             .map(|i| i.name.chars().count())
//             .max()
//             .unwrap_or(0)
//     }

//     async fn set_up(&mut self, ctx: &mut Context) -> Outcome {
//         println!("setting up {} components", self.components.len());
//         let start = Instant::now();
//         let outcome = self.run_component_set_ups(ctx).await;
//         let elapsed = start.elapsed();
//         println!(
//             "\nset up: {}. finished in {:.02}s",
//             outcome,
//             (elapsed.as_millis() as f64) / 1000.0
//         );
//         outcome
//     }

//     async fn run_component_set_ups(&mut self, ctx: &mut Context) -> Outcome {
//         for component in &mut self.components {
//             let start = Instant::now();
//             ctx.log_action_start("set up", &component.name);
//             let outcome = component.set_up(ctx).await;
//             ctx.log_action_end(outcome, start);
//             if outcome != Outcome::Ok {
//                 return Outcome::Failed;
//             } else {
//                 ctx.log_updated_params();
//             }
//         }
//         Outcome::Ok
//     }

//     async fn tear_down(&mut self, ctx: &mut Context) -> Outcome {
//         println!("\ntearing down {} components", self.components.len());
//         let start = Instant::now();
//         let outcome = self.run_component_tear_downs(ctx).await;
//         let elapsed = start.elapsed();
//         println!(
//             "\ntear down: {}. finished in {:.02}s",
//             outcome,
//             (elapsed.as_millis() as f64) / 1000.0
//         );
//         outcome
//     }

//     async fn run_component_tear_downs(&mut self, ctx: &mut Context) -> Outcome {
//         // we attempt to call all tear down functions - even if some fail.
//         let mut all_clean = true;
//         for component in self.components.iter_mut().rev() {
//             let start = Instant::now();
//             ctx.log_action_start("tear down", &component.name);
//             let outcome = component.tear_down().await;
//             ctx.log_action_end(outcome, start);
//             all_clean &= outcome != Outcome::Failed
//         }
//         if all_clean {
//             Outcome::Ok
//         } else {
//             Outcome::Failed
//         }
//     }
// }

fn run_tests() -> Conclusion {
    let args = Arguments::from_args();
    let mut tests = Vec::new();

    for test in inventory::iter::<RegisteredITest> {
        tests.push(Trial::test(test.name.to_owned(), move || {
            (test.test_fn)();
            Ok(())
        }));
    }

    libtest_mimic::run(&args, tests)
}

pub struct ITest {
    context: GlobalContext,
    //components: Components,
}

impl ITest {
    pub fn new() -> Self {
        let workspace_root_dir = find_workspace_root_dir();
        let context = GlobalContext::new(&workspace_root_dir);
        Self {
            context,
            //   components: Components::default(),
        }
    }
}

fn find_workspace_root_dir() -> PathBuf {
    // Get workspace root
    let output = Command::new("cargo")
        .args(&["locate-project", "--workspace", "--message-format=plain"])
        .output()
        .expect("Failed to locate workspace");

    let stdout = output.stdout;
    let workspace_root = String::from_utf8(stdout).expect("Invalid UTF-8");

    let workspace_root = workspace_root.trim().trim_end_matches("/Cargo.toml");

    PathBuf::from(workspace_root).canonicalize().unwrap()
}

impl ITest {
    pub fn set(mut self, key: &str, value: &str) -> Self {
        self.context.set_global_param(key, value);
        self
    }

    pub fn run(self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.run_async())
    }

    async fn run_async(mut self) {
        run_set_ups(&mut self.context).await.unwrap();

        // self.context.max_component_name_len = self.components.max_component_name_len();

        // let set_up_status = self.components.set_up(&mut self.context).await;

        // let conculsion = if set_up_status == Outcome::Ok {
        //     Some(run_tests())
        // } else {
        //     None
        // };

        // let tear_down_status = self.components.tear_down(&mut self.context).await;

        // for component in &self.components.components {
        //     if let Some(err) = &component.set_up_err {
        //         println!("{} set up failed:\n{}", component.name, err);
        //     }
        // }

        // for component in &self.components.components {
        //     if let Some(err) = &component.tear_down_err {
        //         println!("{} tear down failed:\n{}", component.name, err);
        //     }
        // }

        // println!("\nsummary");
        // println!("  set ups: {}", set_up_status);
        // println!("    tests: TBC");
        // println!("tear down: {}", tear_down_status);

        // if let Some(conclusion) = conculsion {
        //     conclusion.exit();
        // }
    }
}
