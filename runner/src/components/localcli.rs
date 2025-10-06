use std::{
    fs::File,
    process::{Command, Stdio},
};

use crate::Context;

pub struct LocalCliSetUp {
    name: String,
    args: Vec<String>,
    envs: Vec<(String, String)>,
}

impl LocalCliSetUp {
    pub fn new(name: &str) -> LocalCliSetUp {
        LocalCliSetUp {
            name: name.to_owned(),
            args: Vec::new(),
            envs: Vec::new(),
        }
    }

    pub fn with_args(self, args: &[&str]) -> LocalCliSetUp {
        LocalCliSetUp {
            name: self.name,
            args: args.iter().map(|i| i.to_string()).collect(),
            envs: self.envs,
        }
    }

    pub fn with_envs(self, envs: &[(&str, &str)]) -> LocalCliSetUp {
        LocalCliSetUp {
            name: self.name,
            args: self.args,
            envs: envs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    pub fn run(self, ctx: Context) -> Result<(), Box<dyn std::error::Error>> {
        let binary = ctx.workspace_binary_path(&self.name);
        let stdout_file = File::create(ctx.log_file_path("stdout"))?;
        let stderr_file = File::create(ctx.log_file_path("stderr"))?;
        let child = Command::new(binary)
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .envs(self.envs.clone())
            .args(&self.args)
            .spawn()?;

        let output = child.wait_with_output()?.exit_ok()?;
        Ok(())
    }
}
