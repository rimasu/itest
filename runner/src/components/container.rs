use std::{
    fs::File,
    io::{self, BufRead, BufWriter, Write},
    path::Path,
};

use async_trait::async_trait;
use testcontainers::{ContainerAsync, GenericImage};

use crate::TearDown;

pub struct ContainerTearDown {
    container: Option<ContainerAsync<GenericImage>>,
}

impl ContainerTearDown {
    pub fn new(container: ContainerAsync<GenericImage>) -> Self {
        Self {
            container: Some(container),
        }
    }
}

#[async_trait]
impl TearDown for ContainerTearDown {
    async fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(container) = self.container.take() {
            container.stop().await?;
            container.rm().await?;
        }

        // // for now just write the logs at the end
        // dump_to_file(&mut stdout, &self.stdout_file)?;
        // dump_to_file(&mut stderr, &self.stderr_file)?;

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
