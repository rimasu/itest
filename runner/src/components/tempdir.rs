use crate::{Context, SetUpResult, TearDown};
use tempfile::TempDir;

pub fn set_up_temp_dir(ctx: &mut Context) -> SetUpResult {
    let temp_dir = TempDir::new()?;
    ctx.set_param("path", temp_dir.path().to_str().unwrap());
    Ok(Box::new(TempDirComponent {
        temp_dir: Some(temp_dir),
    }))
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
