use std::{
    fmt, fs,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};

use dashmap::DashMap;
use tokio::io::AsyncBufRead;

use crate::progress::ProgressListener;

#[derive(Clone)]
pub struct Param {
    raw: String,
}

impl Param {
    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl fmt::Debug for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

pub struct GlobalContext {
    params: Arc<DashMap<String, Param>>,
    workspace_root_dir: PathBuf,
}

impl GlobalContext {
    pub(crate) fn new(workspace_root_dir: &Path) -> GlobalContext {
        let mut log_dir = workspace_root_dir.to_owned();
        log_dir.push("target");
        log_dir.push("itest");
        log_dir.push("logs");
        fs::create_dir_all(&log_dir).unwrap();
        Self {
            params: Arc::new(DashMap::new()),
            workspace_root_dir: workspace_root_dir.to_path_buf(),
        }
    }

    pub(crate) fn create_component_context(&mut self, name: &str) -> Context {
        Context {
            params: self.params.clone(),
            workspace_root_dir: self.workspace_root_dir.clone(),
            component_name: name.to_owned(),
        }
    }

    pub fn set_global_param(&mut self, key: &str, value: &str) {
        self.params.insert(
            key.to_owned(),
            Param {
                raw: value.to_owned(),
            },
        );
    }
}

pub struct Context {
    params: Arc<DashMap<String, Param>>,
    workspace_root_dir: PathBuf,
    component_name: String,
}

impl Context {
    fn clean_component_name(&self) -> String {
        let clean_name = self.component_name.replace("/", "_");
        clean_name.trim().to_string()
    }

    fn log_dir(&self) -> PathBuf {
        let mut log_dir = self.workspace_root_dir.to_owned();
        log_dir.push("target");
        log_dir.push("itest");
        log_dir.push("logs");
        fs::create_dir_all(&log_dir).unwrap();
        log_dir
    }

    pub fn monitor_async(&self, name: &str, mut reader: Pin<Box<dyn AsyncBufRead + Send>>) {
        let path = self.log_file_path(name);
        let handle = tokio::spawn(async move {
            let mut file = tokio::fs::File::create(path).await?;
            tokio::io::copy(&mut reader, &mut file).await?;
            Ok::<_, std::io::Error>(())
        });
    }

    /// Create a path suitable for logging the components output
    ///
    /// If your component only generates one output file you should
    /// put it here.
    pub fn default_log_file_path(&self) -> PathBuf {
        let mut dir = self.log_dir();
        dir.push(format!("{}.log", self.clean_component_name()));
        dir
    }

    /// Create a path suitable for logging the components output
    ///
    /// If your component only generates out output you should use
    /// ```default_log_file_path()``` instead.
    pub fn log_file_path(&self, log_name: &str) -> PathBuf {
        let mut dir = self.log_dir();
        dir.push(format!("{}.{}.log", self.clean_component_name(), log_name));
        dir
    }

    /// Name of a binary file in the workspace
    pub fn workspace_binary_path(&self, binary_name: &str) -> PathBuf {
        let profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        let mut path = self.workspace_root_dir.to_path_buf();
        path.push("target");
        path.push(profile);
        path.push(binary_name);
        path
    }

    pub fn get_param(&self, key: &str) -> Result<Param, ()> {
        self.params.get(key).ok_or(()).map(|p| p.clone())
    }

    pub fn set_param(&self, key: &str, value: &str) {
        let key = format!("{}.{}", self.clean_component_name(), key);
        let param = Param {
            raw: value.to_owned(),
        };
        self.params.insert(key.to_owned(), param.clone());
    }
}
