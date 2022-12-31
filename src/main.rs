extern crate core;

mod cli;
mod command;
mod config;

use std::process::ExitStatus;

use anyhow::{anyhow, Result};
use clap::Parser;
use command::*;
use log::debug;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::cli::Args;
use crate::config::Config;

#[allow(dead_code)]
fn expand(param: impl AsRef<str>) -> String {
    let parts: Vec<&str> = param.as_ref().splitn(2, ':').collect();
    match parts.as_slice() {
        ["cargo", cmd] => format!("cargo run --color always --package {}", cmd),
        _ => param.as_ref().to_string(),
    }
}

async fn run(config: &Config, tx: mpsc::Sender<Event>) -> Result<()> {
    let mut joins = JoinSet::new();
    let mut pids = vec![];

    for (idx, cmd) in config.commands.iter().enumerate() {
        let mut runnable: tokio::process::Command = cmd.into();
        let mut child = runnable.spawn()?;

        let pid= child.id()
            .expect("Successfully spawned child should have a PID");
        pids.push(pid);

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to acquire stdout handle"))?;

        let tx2 = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();

            while let Some(line) = reader.next_line().await.unwrap() {
                tx2.send(Event::Output {
                    line,
                    command_idx: idx,
                })
                .await
                .unwrap();
            }
        });

        joins.spawn(async move { (idx, child.wait().await.unwrap()) });
    }

    while let Some(res) = joins.join_next().await {
        let (idx, status) = res?;

        tx.send(Event::Exit {
            status,
            command_idx: idx,
        })
        .await
        .unwrap();
    }

    Ok(())
}

#[derive(Debug)]
enum Event {
    Output {
        line: String,
        command_idx: usize,
    },
    Exit {
        status: ExitStatus,
        command_idx: usize,
    },
}

const OUTPUT_CHANNEL_BUFFER_SIZE: usize = 128;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let args: Args = Args::parse();
    let config: &'static Config = Box::leak(Box::new(args.try_into()?));
    debug!("{:#?}", config);

    // This is the channel that is used to communicate everything that's happening
    // in the spawned processes back here, we're output is handled.
    let (tx, mut rx) = mpsc::channel::<Event>(OUTPUT_CHANNEL_BUFFER_SIZE);

    let handle = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                Event::Output { command_idx, line } => {
                    let cmd = config.commands.get(command_idx).unwrap();
                    println!("[{}] {}", cmd.name, line);
                }
                Event::Exit {
                    command_idx,
                    status,
                } => {
                    let cmd = config.commands.get(command_idx).unwrap();
                    let full_command = config.args.commands.get(command_idx).unwrap();
                    println!(
                        "[{}] {} exited with code {}",
                        cmd.name,
                        full_command,
                        status.code().unwrap()
                    );
                }
            }
        }
    });

    run(config, tx).await?;
    handle.await?;
    Ok(())
}
