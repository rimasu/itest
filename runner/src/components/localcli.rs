use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
    process::{Command, Output, Stdio},
};

use async_trait::async_trait;

use crate::{AsyncSetUp, Context, SetUpResult, TearDown};

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
}


#[async_trait]
impl AsyncSetUp for LocalCliSetUp {
    async fn set_up(&mut self, ctx: &mut Context) -> SetUpResult {
        let binary = ctx.workspace_binary_path(&self.name);
        let mut child = Command::new(binary)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(self.envs.clone())
            .args(&self.args)
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let output = child.wait_with_output()?.exit_ok();

        let stdout_file = ctx.log_file_path("stdout");
        let stderr_file = ctx.log_file_path("stderr");

        let mut stdout = BufReader::new(stdout);
        dump_to_file(&mut stdout, &stdout_file).unwrap();

        let mut stderr = BufReader::new(stderr);
        dump_to_file(&mut stderr, &stderr_file).unwrap();

        Ok(Box::new(LocalCliComponent {
            name: self.name.to_owned(),
            output: output?,
        }))
    }
}

pub struct LocalCliComponent {
    name: String,
    output: Output,
}

#[async_trait]
impl TearDown for LocalCliComponent {
    async fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

fn dump_to_file<R>(mut reader: &mut R, file_path: &Path) -> io::Result<()>
where
    R: BufRead,
{
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        writer.write_all(line.as_bytes())?;
        line.clear();
    }

    writer.flush()?;
    Ok(())
}
