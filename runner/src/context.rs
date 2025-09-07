use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub struct Param {
    raw: String,
}

impl fmt::Debug for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

#[derive(Default)]
pub struct Context {
    params: BTreeMap<String, Param>,
    updated_params: BTreeSet<String>,
}

impl Context {
    pub fn get_param(&self, key: &str) -> Result<&Param, ()> {
        self.params.get(key).ok_or(())
    }

    pub fn set_param(&mut self, key: &str, value: &str) {
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
