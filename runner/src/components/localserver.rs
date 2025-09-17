use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

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
        let child = Command::new(binary)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout_file = ctx.log_file_path("stdout");
        let stderr_file = ctx.log_file_path("stderr");

        Ok(Box::new(LocalRunnerComponent {
            name: self.name.to_owned(),
            child,
            stdout_file,
            stderr_file,
        }))
    }
}

pub struct LocalRunnerComponent {
    name: String,
    child: Child,
    stdout_file: PathBuf,
    stderr_file: PathBuf,
}

impl TearDown for LocalRunnerComponent {
    fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(stdout) = self.child.stdout.take() {
            let mut stdout = BufReader::new(stdout);
            dump_to_file(&mut stdout, &self.stdout_file).unwrap();
        }

        if let Some(stderr) = self.child.stderr.take() {
            let mut stderr = BufReader::new(stderr);
            dump_to_file(&mut stderr, &self.stderr_file).unwrap();
        }

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
