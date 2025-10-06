use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use async_trait::async_trait;

use crate::{Context, TearDown};

// use async_trait::async_trait;

// use crate::{AsyncSetUp, Context, SetUpResult, TearDown};

pub struct LocalServerSetUp {
    name: String,
    args: Vec<String>,
    envs: Vec<(String, String)>,
}

impl LocalServerSetUp {
    pub fn new(name: &str) -> LocalServerSetUp {
        LocalServerSetUp {
            name: name.to_owned(),
            args: Vec::new(),
            envs: Vec::new(),
        }
    }

    pub fn with_args(self, args: &[&str]) -> LocalServerSetUp {
        LocalServerSetUp {
            name: self.name,
            args: args.iter().map(|i| i.to_string()).collect(),
            envs: self.envs,
        }
    }

    pub fn with_envs(self, envs: &[(&str, &str)]) -> LocalServerSetUp {
        LocalServerSetUp {
            name: self.name,
            args: self.args,
            envs: envs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    pub fn start(self, ctx: Context) -> Result<impl TearDown, Box<dyn std::error::Error>> {
        let binary = ctx.workspace_binary_path(&self.name);

        let stdout_file = File::create(ctx.log_file_path("stdout"))?;
        let stderr_file = File::create(ctx.log_file_path("stderr"))?;

        let child = Command::new(binary)
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .envs(self.envs.clone())
            .args(&self.args)
            .spawn()?;

        Ok(LocalRunnerComponent { child })
    }
}

pub struct LocalRunnerComponent {
    child: Child,
}

#[async_trait]
impl TearDown for LocalRunnerComponent {
    async fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.child.kill()?;

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
