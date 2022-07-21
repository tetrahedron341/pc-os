use argh::FromArgs;
use std::path::PathBuf;

#[derive(FromArgs, Debug, Clone)]
/// Build system for this currently unnamed kernel
pub struct Args {
    /// path to kbuild config (default is ./kbuild.toml)
    #[argh(option, default = "\"./kbuild.toml\".into()")]
    pub config_path: PathBuf,

    #[argh(subcommand)]
    pub command: Command,
}

#[derive(FromArgs, Debug, Clone)]
#[argh(subcommand)]
pub enum Command {
    Build(BuildCommand),
}

/// Build an image
#[derive(FromArgs, Debug, Clone)]
#[argh(subcommand, name = "build")]
pub struct BuildCommand {
    /// path to put the built image (default: ./out.img)
    #[argh(option, default = "\"./out.img\".into()")]
    pub out_path: PathBuf,
}
