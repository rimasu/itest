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
    fn set_up(&mut self, ctx: &mut Context) -> SetUpResult {
        let binary = ctx.workspace_binary_path(&self.name);
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

