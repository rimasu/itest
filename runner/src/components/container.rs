use std::result::Result;

use testcontainers::{ContainerRequest, GenericImage, runners::AsyncRunner};

// pub async fn set_up_container(
//     req: ContainerRequest<GenericImage>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let container = self.req.take().unwrap().start().await?;
//     Ok(())
// }

// #[async_trait]
// impl TearDown for ContainerComponent {
//     async fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
//         if let Some(container) = self.container.take() {
//             container.stop().await?;
//             container.rm().await?;
//         }

//         // // for now just write the logs at the end
//         // dump_to_file(&mut self.stdout, &self.stdout_file).unwrap();
//         // dump_to_file(&mut self.stderr, &self.stderr_file).unwrap();
//         Ok(())
//     }
// }

// fn dump_to_file(reader: &mut Box<dyn BufRead + Send>, file_path: &Path) -> io::Result<()> {
//     let file = File::create(file_path)?;
//     let mut writer = BufWriter::new(file);

//     let mut line = String::new();
//     while reader.read_line(&mut line)? > 0 {
//         writer.write_all(line.as_bytes())?;
//         line.clear();
//     }

//     writer.flush()?;
//     Ok(())
// }
