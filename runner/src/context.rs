use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use crate::Outcome;

pub struct Param {
    raw: String,
}

impl fmt::Debug for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

pub struct Context {
    params: BTreeMap<String, Param>,
    updated_params: BTreeSet<String>,
    workspace_root_dir: PathBuf,
    pub max_component_name_len: usize,
    current_component_name: String,
}

impl Context {
    pub(crate) fn new(workspace_root_dir: &Path) -> Context {
        let mut log_dir = workspace_root_dir.to_owned();
        log_dir.push("target");
        log_dir.push("itest");
        log_dir.push("logs");
        fs::create_dir_all(&log_dir).unwrap();
        Self {
            params: BTreeMap::new(),
            updated_params: BTreeSet::new(),
            workspace_root_dir: workspace_root_dir.to_path_buf(),
            max_component_name_len: 0,
            current_component_name: "".to_owned(),
        }
    }

    pub(crate) fn set_current_component(&mut self, name: &str) {
        self.current_component_name = name.to_owned();
    }

    fn clean_component_name(&self) -> String {
        let clean_name = self.current_component_name.replace("/", "_");
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

    pub(crate) fn log_action_start(&self, action: &str, name: &str) {
        print!(
            "{} {:width$} ... ",
            action,
            name,
            width = self.max_component_name_len
        );
        io::stdout().flush().unwrap();
    }

    pub(crate) fn log_action_end(&self, status: Outcome, start: Instant) {
        if status == Outcome::Skipped {
            println!("{}", status);
        } else {
            let elapsed = start.elapsed();
            println!(
                "{} ({:.02}s)",
                status,
                (elapsed.as_millis() as f64) / 1000.0
            );
        }
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

    pub fn get_param(&self, key: &str) -> Result<&Param, ()> {
        self.params.get(key).ok_or(())
    }

    pub fn set_global_param(&mut self, key: &str, value: &str) {
        self.params.insert(
            key.to_owned(),
            Param {
                raw: value.to_owned(),
            },
        );
    }

    pub fn set_param(&mut self, key: &str, value: &str) {
        let key = format!("{}.{}", self.clean_component_name(), key);
        self.params.insert(
            key.to_owned(),
            Param {
                raw: value.to_owned(),
            },
        );
        self.updated_params.insert(key.to_owned());
    }

    pub(crate) fn log_updated_params(&mut self) {
        for updated_param in &self.updated_params {
            let value = self
                .params
                .get(updated_param)
                .map(|p| p.raw.as_str())
                .unwrap_or("");

            println!("      {} = {}", updated_param, value);
        }
        if !&self.updated_params.is_empty() {
            println!();
        }
        self.updated_params.clear();
    }
}
