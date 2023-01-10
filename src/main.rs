extern crate core;

mod cli;
mod colors;
mod command;
mod config;
mod event_loop;

use anyhow::{bail, Result};
use clap::{CommandFactory, Parser};
use command::*;
use log::debug;

use crate::cli::Args;
use crate::config::Config;
use crate::event_loop::event_loop;

#[allow(dead_code)]
fn expand(param: impl AsRef<str>) -> String {
    let parts: Vec<&str> = param.as_ref().splitn(2, ':').collect();
    match parts.as_slice() {
        ["cargo", cmd] => format!("cargo run --color always --package {}", cmd),
        _ => param.as_ref().to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let args: Args = Args::parse();
    let config: &'static Config = Box::leak(Box::new(args.try_into()?));
    debug!("{:#?}", config);

    if config.commands.is_empty() {
        Args::command().print_long_help()?;
        println!();
        bail!("No commands were given");
    }

    event_loop(config).await
}
