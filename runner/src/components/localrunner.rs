use std::{
    path::PathBuf,
    process::{Child, Command},
};

use crate::{Context, SetUp, SetUpResult, TearDown};

pub struct LocalRunnerSetUp {
    name: String,
}

impl LocalRunnerSetUp {
    pub fn new(name: &str) -> Box<LocalRunnerSetUp> {
        Box::new(LocalRunnerSetUp {
            name: name.to_owned(),
        })
    }
}

impl SetUp for LocalRunnerSetUp {
    fn set_up(&mut self, _ctx: &mut Context) -> SetUpResult {
        let binary = get_binary_path(&self.name);
        let child = Command::new(binary).spawn()?;

        Ok(Box::new(LocalRunnerComponent {
            name: self.name.to_owned(),
            child,
        }))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct LocalRunnerComponent {
    name: String,
    child: Child,
}

impl TearDown for LocalRunnerComponent {
    fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.child.kill()?;
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
