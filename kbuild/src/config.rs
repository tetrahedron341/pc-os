use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// Path to kernel crate directory relative to project root
    pub kernel_path: PathBuf,

    /// Target to build by default
    pub default_target: String,

    /// Target-specific keys
    #[serde(rename = "target")]
    pub target_specific: HashMap<String, TargetDescriptor>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TargetDescriptor {
    /// Either a target triple or a path to a .json file
    /// (https://doc.rust-lang.org/nightly/rustc/targets/custom.html)
    pub target_spec: String,
}
