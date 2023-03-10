#![doc = include_str!("../README.md")]
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

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let args: Args = Args::parse();
    if args.kill_others && args.kill_others_on_fail {
        bail!(
            "\nYou supplied both --kill-others and --kill-others-on-fail,\
             \nbut the the latter is implied by the former. Please only \
             \nprovide one of these options"
        )
    }

    let config: &'static Config = Box::leak(Box::new(args.try_into()?));
    debug!("{:#?}", config);

    if config.commands.is_empty() {
        Args::command().print_long_help()?;
        println!();
        bail!("No commands were given");
    }

    event_loop(config).await
}
