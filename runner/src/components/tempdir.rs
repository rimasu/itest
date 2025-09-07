use crate::{Context, SetUp, SetUpResult, TearDown};
use tempfile::TempDir;


pub struct TempDirSetUp {
    name: String,
}

impl TempDirSetUp {
    pub fn new(name: &str) -> Box<TempDirSetUp> {
        Box::new(TempDirSetUp {
            name: name.to_owned(),
        })
    }
}

impl SetUp for TempDirSetUp {
    fn set_up(&mut self, ctx: &mut Context) -> SetUpResult {
        let temp_dir = TempDir::new()?;
        let key = format!("{}.path", self.name);
        ctx.set_param(&key, temp_dir.path().to_str().unwrap());
        Ok(Box::new(TempDirComponent {
            temp_dir: Some(temp_dir),
        }))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

struct TempDirComponent {
    temp_dir: Option<TempDir>,
}

impl TearDown for TempDirComponent {
    fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.temp_dir.take().unwrap().close()?;
        Ok(())
    }
}
