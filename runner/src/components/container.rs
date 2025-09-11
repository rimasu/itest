use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

use testcontainers::{Container, ContainerRequest, GenericImage, Image, runners::SyncRunner};

use crate::{Context, SetUp, SetUpResult, TearDown};

pub struct ContainerSetUp {
    name: String,
    image: Option<ContainerRequest<GenericImage>>,
}

impl ContainerSetUp {
    pub fn new(request: ContainerRequest<GenericImage>) -> Box<ContainerSetUp> {
        let name = request.image().name().to_owned();
        Box::new(ContainerSetUp {
            name,
            image: Some(request),
        })
    }
}

impl SetUp for ContainerSetUp {
    fn set_up(&mut self, ctx: &mut Context) -> SetUpResult {
        let image = self.image.take().unwrap();
        let container = image.start()?;
        let stdout_file = ctx.log_file_path("stdout");
        let stderr_file = ctx.log_file_path("stderr");
        let stdout = container.stdout(true);
        let stderr = container.stderr(true);
        Ok(Box::new(ContainerComponent {
            container,
            stdout,
            stdout_file,
            stderr,
            stderr_file,
        }))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct ContainerComponent {
    container: Container<GenericImage>,
    stdout: Box<dyn BufRead + Send>,
    stdout_file: PathBuf,
    stderr: Box<dyn BufRead + Send>,
    stderr_file: PathBuf,
}

impl TearDown for ContainerComponent {
    fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.container.stop()?;

        // for now just write the logs at the end
        dump_to_file(&mut self.stdout, &self.stdout_file).unwrap();
        dump_to_file(&mut self.stderr, &self.stderr_file).unwrap();
        Ok(())
    }
}

fn dump_to_file(reader: &mut Box<dyn BufRead + Send>, file_path: &Path) -> io::Result<()> {
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
