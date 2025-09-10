use std::{
    io,
    path::PathBuf,
    process::{Command, Output},
};

use crate::{Context, SetUp, SetUpResult, TearDown};

pub struct LocalCliSetUp {
    name: String,
    args: Vec<String>,
}

impl LocalCliSetUp {
    pub fn new(name: &str) -> Box<LocalCliSetUp> {
        Box::new(LocalCliSetUp {
            name: name.to_owned(),
            args: Vec::new(),
        })
    }

    pub fn with_args(self, args: &[&str]) -> Box<LocalCliSetUp> {
        Box::new(LocalCliSetUp {
            name: self.name,
            args: args.iter().map(|i| i.to_string()).collect(),
        })
    }
}

impl SetUp for LocalCliSetUp {
    fn set_up(&mut self, _ctx: &mut Context) -> SetUpResult {
        let binary = get_binary_path(&self.name);
        let child = Command::new(binary).args(&self.args).spawn()?;

        let output = child.wait_with_output()?.exit_ok()?;

        Ok(Box::new(LocalCliComponent {
            name: self.name.to_owned(),
            output,
        }))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct LocalCliComponent {
    name: String,
    output: Output,
}

impl TearDown for LocalCliComponent {
    fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

fn get_binary_path(name: &str) -> PathBuf {
    // Option 2: Construct from cargo metadata
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    // Get workspace root
    let output = Command::new("cargo")
        .args(&["locate-project", "--workspace", "--message-format=plain"])
        .output()
        .expect("Failed to locate workspace");

    let stdout = output.stdout;
    let workspace_root = String::from_utf8(stdout).expect("Invalid UTF-8");

    let workspace_root = workspace_root.trim().trim_end_matches("/Cargo.toml");

    PathBuf::from(workspace_root)
        .join("target")
        .join(profile)
        .join(name)
}
