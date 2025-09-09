use std::io::BufRead;

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
    fn set_up(&mut self, _ctx: &mut Context) -> SetUpResult {
        let image = self.image.take().unwrap();
        let container = image.start()?;
        let stdout = container.stdout(true);
        let stderr = container.stderr(true);
        Ok(Box::new(ContainerComponent {
            name: self.name.to_owned(),
            container,
            stdout,
            stderr,
        }))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct ContainerComponent {
    name: String,
    container: Container<GenericImage>,
    stdout: Box<dyn BufRead + Send>,
    stderr: Box<dyn BufRead + Send>,
}

impl TearDown for ContainerComponent {
    fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.container.stop()?;

        // for now just write the logs at the end

        let clean_name = self.name.replace("/", "_");

        let stdout_name = format!("/tmp/{}.stdout.log", &clean_name);
        dump_to_file(&mut self.stdout, &stdout_name).unwrap();

        let stderr_name = format!("/tmp/{}.stderr.log", &clean_name);
        dump_to_file(&mut self.stderr, &stderr_name).unwrap();

        Ok(())
    }
}

use std::fs::File;
use std::io::{self, BufWriter, Write};

fn dump_to_file(reader: &mut Box<dyn BufRead + Send>, file_path: &str) -> io::Result<()> {
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
