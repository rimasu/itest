#![feature(exit_status_error)]

use std::{
    fmt,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use std::time::Instant;

pub use inventory::{collect, submit};
pub use itest_macros::itest;

pub mod components;

mod context;

pub use context::{Context, Param};

use libtest_mimic::{Arguments, Conclusion, Trial};

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

pub struct RegisteredITest {
    pub name: &'static str,
    pub test_fn: fn(),
}
inventory::collect!(RegisteredITest);

pub type SetUpResult = Result<Box<dyn TearDown + 'static>, Box<dyn std::error::Error>>;

pub trait SetUp {
    fn set_up(&mut self, ctx: &mut Context) -> SetUpResult;
    fn name(&self) -> &str;
}

pub trait TearDown {
    fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

struct Component<'s> {
    max_name_len: usize,
    set_up: &'s mut Box<dyn SetUp>,
    set_up_err: Option<Box<dyn std::error::Error>>,
    tear_down: Option<Box<dyn TearDown + 'static>>,
    tear_down_err: Option<Box<dyn std::error::Error>>,
}

impl<'s> Component<'s> {
    fn new(set_up: &'s mut Box<dyn SetUp>, max_name_len: usize) -> Component<'s> {
        Component {
            max_name_len,
            set_up,
            set_up_err: None,
            tear_down: None,
            tear_down_err: None,
        }
    }

    fn log_action_start(&self, action: &str) {
        print!(
            "{} {:width$} ... ",
            action,
            &self.set_up.name(),
            width = self.max_name_len
        );
        io::stdout().flush().unwrap();
    }

    fn log_action_end(&self, status: Outcome, start: Instant) {
        if status == Outcome::Skipped {
            println!("{}", status);
        } else {
            let elapsed = start.elapsed();
            println!(
                "{} ({:.02}s)",
                status,
                (elapsed.as_millis() as f64) / 1000.0
            );
        }
    }

    fn set_up(&mut self, ctx: &mut Context) -> Outcome {
        self.log_action_start("set up");

        let start = Instant::now();
        ctx.set_current_component(self.set_up.name());

        let outcome = match self.set_up.set_up(ctx) {
            Ok(tear_down) => {
                self.tear_down = Some(tear_down);
                Outcome::Ok
            }
            Err(err) => {
                self.set_up_err = Some(err);
                Outcome::Failed
            }
        };

        self.log_action_end(outcome, start);

        outcome
    }

    fn tear_down(&mut self) -> Outcome {
        self.log_action_start("tear down");
        let start = Instant::now();
        let outcome = if let Some(tear_down) = &mut self.tear_down {
            match tear_down.tear_down() {
                Ok(()) => Outcome::Ok,
                Err(err) => {
                    self.tear_down_err = Some(err);
                    Outcome::Failed
                }
            }
        } else {
            Outcome::Skipped
        };
        self.log_action_end(outcome, start);
        outcome
    }
}

struct Components<'s> {
    components: Vec<Component<'s>>,
}

impl<'s> Components<'s> {
    pub fn new(components: Vec<Component<'s>>) -> Components<'s> {
        Self { components }
    }
}

impl<'s> Components<'s> {
    fn set_up(&mut self, ctx: &mut Context) -> Outcome {
        println!("setting up {} components", self.components.len());
        let start = Instant::now();
        let outcome = self.run_component_set_ups(ctx);
        let elapsed = start.elapsed();
        println!(
            "\nset up: {}. finished in {:.02}s",
            outcome,
            (elapsed.as_millis() as f64) / 1000.0
        );
        outcome
    }

    fn run_component_set_ups(&mut self, ctx: &mut Context) -> Outcome {
        for component in &mut self.components {
            if component.set_up(ctx) != Outcome::Ok {
                return Outcome::Failed;
            } else {
                ctx.log_updated_params();
            }
        }
        Outcome::Ok
    }

    fn tear_down(&mut self) -> Outcome {
        println!("\ntearing down {} components", self.components.len());
        let start = Instant::now();
        let outcome = self.run_component_tear_downs();
        let elapsed = start.elapsed();
        println!(
            "\ntear down: {}. finished in {:.02}s",
            outcome,
            (elapsed.as_millis() as f64) / 1000.0
        );
        outcome
    }

    fn run_component_tear_downs(&mut self) -> Outcome {
        // we attempt to call all tear down functions - even if some fail.
        let mut all_clean = true;
        for component in self.components.iter_mut().rev() {
            all_clean &= component.tear_down() != Outcome::Failed
        }
        if all_clean {
            Outcome::Ok
        } else {
            Outcome::Failed
        }
    }
}

fn max_set_up_name_len(set_ups: &[Box<dyn SetUp>]) -> usize {
    let mut max_len = 0;
    for set_up in set_ups {
        max_len = max_len.max(set_up.name().chars().count());
    }
    max_len
}

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

fn make_components<'s>(set_ups: &'s mut [Box<dyn SetUp>]) -> Components<'s> {
    let max_name_len = max_set_up_name_len(set_ups);

    let components = set_ups
        .into_iter()
        .map(|s| Component::new(s, max_name_len))
        .collect();

    Components::new(components)
}

pub struct ITest {
    context: Context,
    set_ups: Vec<Box<dyn SetUp>>,
}

impl ITest {
    pub fn new() -> Self {
        let workspace_root_dir = find_workspace_root_dir();
        let context = Context::new(&workspace_root_dir);
        Self {
            context,
            set_ups: Vec::new(),
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
        self.context.set_param(key, value);
        self
    }

    pub fn with(mut self, set_up: Box<dyn SetUp>) -> Self {
        self.set_ups.push(set_up);
        self
    }

    pub fn run(mut self) {
        let mut components = make_components(&mut self.set_ups);

        let set_up_status = components.set_up(&mut self.context);

        let conculsion = if set_up_status == Outcome::Ok {
            Some(run_tests())
        } else {
            None
        };

        let tear_down_status = components.tear_down();

        for component in &components.components {
            if let Some(err) = &component.set_up_err {
                println!("{} set up failed:\n{}", component.set_up.name(), err);
            }
        }

        for component in &components.components {
            if let Some(err) = &component.tear_down_err {
                println!("{} tear down failed:\n{}", component.set_up.name(), err);
            }
        }

        println!("\nsummary");
        println!("  set ups: {}", set_up_status);
        println!("    tests: TBC");
        println!("tear down: {}", tear_down_status);

        if let Some(conclusion) = conculsion {
            conclusion.exit();
        }
    }
}
