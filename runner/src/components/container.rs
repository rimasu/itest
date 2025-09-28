use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::Path;
use std::result::Result;

use async_trait::async_trait;
use testcontainers::ContainerAsync;
use testcontainers::{ContainerRequest, GenericImage, runners::AsyncRunner};

use crate::{AsyncSetUp, Context, SetUpResult, TearDown};

pub fn set_up_container(
    req: ContainerRequest<GenericImage>,
) -> Result<Box<dyn AsyncSetUp>, Box<dyn std::error::Error>> {
    Ok(Box::new(ContainerSetUp { req: Some(req) }))
}

struct ContainerSetUp {
    req: Option<ContainerRequest<GenericImage>>,
}

#[async_trait]
impl AsyncSetUp for ContainerSetUp {
    async fn set_up(&mut self, ctx: &mut Context) -> SetUpResult {
        let container = self.req.take().unwrap().start().await?;
        Ok(Box::new(ContainerComponent {
            container: Some(container),
        }))
    }
}

pub struct ContainerComponent {
    container: Option<ContainerAsync<GenericImage>>,
}

#[async_trait]
impl TearDown for ContainerComponent {
    async fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(container) = self.container.take() {
            container.stop().await?;
            container.rm().await?;
        }

        // // for now just write the logs at the end
        // dump_to_file(&mut self.stdout, &self.stdout_file).unwrap();
        // dump_to_file(&mut self.stderr, &self.stderr_file).unwrap();
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
