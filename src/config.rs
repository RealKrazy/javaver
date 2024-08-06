use std::{fs::File, io::{self, Read, Write}, path::PathBuf};

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct JavaverConfig {
    pub sdk: Vec<SDKConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SDKConfig {
    pub name: String,
    pub path: PathBuf,
}

impl JavaverConfig {
    pub fn new() -> Self {
        Self {
            sdk: Vec::new(),
        }
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.sdk.iter().find(|el| el.name == name).is_some()
    }
}

impl SDKConfig {
    pub fn new(name: &String, path: &PathBuf) -> Self {
        Self {
            name: name.clone(),
            path: path.clone(),
        }
    }
}

pub type WriteConfigResult = io::Result<()>;
pub type ReadConfigResult = io::Result<JavaverConfig>;

pub fn write_config(config: &JavaverConfig, path: &PathBuf) -> WriteConfigResult {
    let serialized = serde_json::to_string(config).unwrap();

    let mut file = File::create(path)?;
    file.write(serialized.as_bytes())?;

    Ok(())
}

pub fn read_config(path: &PathBuf) -> ReadConfigResult {
    let mut file = File::open(path)?;
    
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let config: JavaverConfig = serde_json::from_slice(&buf.as_slice()).unwrap();

    Ok(config)
}