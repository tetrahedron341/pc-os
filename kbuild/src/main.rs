#![feature(try_blocks)]
use color_eyre::{eyre::ContextCompat, Result};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

mod args;
mod config;
use args::{Args, BuildCommand, Command};
use config::Config;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args: Args = argh::from_env();

    match args.command {
        Command::Build(build_args) => build_img(args.config_path, build_args),
    }
}

fn parse_project_config(config_path: &Path) -> Result<Config> {
    use serde::Deserialize;

    let config_toml = toml::Value::from_str(&std::fs::read_to_string(config_path)?)?;
    let table = config_toml.get("kbuild").ok_or_else(|| {
        color_eyre::Report::msg(
            "Couldn't find table `workspace.metadata.kbuild` in project manifest",
        )
    })?;
    Ok(Config::deserialize(table.clone())?)
}

fn build_img(config_path: PathBuf, build_args: BuildCommand) -> Result<()> {
    build_kernel(config_path)
}

fn build_kernel(config_path: PathBuf) -> Result<()> {
    let config = parse_project_config(&config_path)?;

    let target_key = config.default_target;
    let target = config
        .target_specific
        .get(&target_key)
        .wrap_err(format!("Target {target_key} does not exist"))?;
    let target_triple = {
        let spec = &target.target_spec;
        if spec.ends_with(".json") {
            config_path.parent().unwrap().join(spec)
        } else {
            spec.into()
        }
    };

    let kernel_path = config_path.parent().unwrap().join(config.kernel_path);
    let cmd = std::process::Command::new("cargo")
        .arg("kbuild")
        .args(["--target", target_triple.to_str().unwrap()])
        .args([
            "-Zbuild-std=core,alloc",
            "-Zbuild-std-features=compiler-builtins-mem",
        ])
        .current_dir(kernel_path);

    Ok(())
}
