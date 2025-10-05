// use std::{
//     fs::File,
//     io::{self, BufRead, BufReader, BufWriter, Read, Write},
//     path::{Path, PathBuf},
//     process::{Child, Command, Stdio},
// };

// use async_trait::async_trait;

// use crate::{AsyncSetUp, Context, SetUpResult, TearDown};

// pub struct LocalServerSetUp {
//     name: String,
//     args: Vec<String>,
//     envs: Vec<(String, String)>,
// }

// impl LocalServerSetUp {
//     pub fn new(name: &str) -> LocalServerSetUp {
//         LocalServerSetUp {
//             name: name.to_owned(),
//             args: Vec::new(),
//             envs: Vec::new(),
//         }
//     }

//     pub fn with_args(self, args: &[&str]) -> LocalServerSetUp {
//         LocalServerSetUp {
//             name: self.name,
//             args: args.iter().map(|i| i.to_string()).collect(),
//             envs: self.envs,
//         }
//     }

//     pub fn with_envs(self, envs: &[(&str, &str)]) -> LocalServerSetUp {
//         LocalServerSetUp {
//             name: self.name,
//             args: self.args,
//             envs: envs
//                 .iter()
//                 .map(|(k, v)| (k.to_string(), v.to_string()))
//                 .collect(),
//         }
//     }
// }

// #[async_trait]
// impl AsyncSetUp for LocalServerSetUp {
//     async fn set_up(&mut self, ctx: &mut Context) -> SetUpResult {
//         let binary = ctx.workspace_binary_path(&self.name);
//         let child = Command::new(binary)
//             .stdout(Stdio::piped())
//             .stderr(Stdio::piped())
//             .envs(self.envs.clone())
//             .args(&self.args)
//             .spawn()?;

//         let stdout_file = ctx.log_file_path("stdout");
//         let stderr_file = ctx.log_file_path("stderr");

//         Ok(Box::new(LocalRunnerComponent {
//             name: self.name.to_owned(),
//             child,
//             stdout_file,
//             stderr_file,
//         }))
//     }
// }

// pub struct LocalRunnerComponent {
//     name: String,
//     child: Child,
//     stdout_file: PathBuf,
//     stderr_file: PathBuf,
// }

// #[async_trait]
// impl TearDown for LocalRunnerComponent {
//     async fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
//         if let Some(stdout) = self.child.stdout.take() {
//             let mut stdout = BufReader::new(stdout);
//             dump_to_file(&mut stdout, &self.stdout_file).unwrap();
//         }

//         if let Some(stderr) = self.child.stderr.take() {
//             let mut stderr = BufReader::new(stderr);
//             dump_to_file(&mut stderr, &self.stderr_file).unwrap();
//         }

//         self.child.kill()?;

//         Ok(())
//     }
// }

// fn dump_to_file<R>(mut reader: &mut R, file_path: &Path) -> io::Result<()>
// where
//     R: BufRead,
// {
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
