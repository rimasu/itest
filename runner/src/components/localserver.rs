use std::process::{Child, Command};

use crate::{Context, SetUpResult, TearDown};

pub struct LocalServerSetUp {
    name: String,
}

impl LocalServerSetUp {
    pub fn new(name: &str) -> Box<LocalServerSetUp> {
        Box::new(LocalServerSetUp {
            name: name.to_owned(),
        })
    }

    pub fn launch(&mut self, ctx: &mut Context) -> SetUpResult {
        let binary = ctx.workspace_binary_path(&self.name);
        let child = Command::new(binary).spawn()?;

        Ok(Box::new(LocalRunnerComponent {
            name: self.name.to_owned(),
            child,
        }))
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
