extern crate core;

mod cli;
mod colors;
mod command;
mod config;

use std::fmt::Debug;
use std::process::ExitStatus;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{anyhow, bail, Context, Result};
use clap::{CommandFactory, Parser};
use command::*;
use log::{debug, error, trace};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
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

#[derive(Debug)]
enum Event {
    Spawn {
        command_idx: usize,
        is_restart: bool,
    },
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

async fn event_loop(
    config: &'static Config,
    tx_orig: Sender<Event>,
    mut rx: Receiver<Event>,
) -> Result<()> {
    let processes_alive = AtomicUsize::new(0);
    let mut processes: JoinSet<Result<()>> = JoinSet::new();

    for (command_idx, _) in config.commands.iter().enumerate() {
        tx_orig
            .send(Event::Spawn {
                command_idx,
                is_restart: false,
            })
            .await
            .unwrap()
    }

    while let Some(event) = rx.recv().await {
        match event {
            Event::Spawn {
                command_idx,
                is_restart,
            } => {
                trace!("Spawning command {command_idx}");

                let cmd = config.commands.get(command_idx).unwrap();
                let mut child = cmd.tokio_command().spawn().expect("Failed to spawn child");

                let pid = child
                    .id()
                    .expect("Successfully spawned child should have a PID");
                cmd.pid.store(pid, Ordering::Relaxed);

                let stdout = child
                    .stdout
                    .take()
                    .ok_or_else(|| anyhow!("Failed to acquire stdout handle"))?;

                let stderr = child
                    .stderr
                    .take()
                    .ok_or_else(|| anyhow!("Failed to acquire stderr handle"))?;

                // This is the task that handles the child's stdout
                let tx = tx_orig.clone();
                processes.spawn(async move {
                    trace!("stdout reader task for {pid} started");

                    let mut reader = BufReader::new(stdout).lines();
                    while let Some(line) = reader.next_line().await? {
                        tx.send(Event::Output { line, command_idx }).await?
                    }

                    trace!("stdout reader task for {pid} stopped");
                    Ok(())
                });

                // This is the task that handles the child's stderr
                let tx = tx_orig.clone();
                processes.spawn(async move {
                    trace!("stderr reader task for {pid} started");

                    let mut reader = BufReader::new(stderr).lines();
                    while let Some(line) = reader.next_line().await? {
                        tx.send(Event::Output { line, command_idx }).await?
                    }

                    trace!("stderr reader task for {pid} stopped");
                    Ok(())
                });

                // This is the task that waits for the child's exit status
                let tx = tx_orig.clone();
                processes.spawn(async move {
                    let status = child.wait().await?;
                    trace!("Task with pid {pid} stopped with {status}");
                    tx.send(Event::Exit {
                        command_idx,
                        status,
                    })
                    .await?;

                    Ok(())
                });

                processes_alive.fetch_add(1, Ordering::SeqCst);
                if is_restart {
                    let full_command = config.args.commands.get(command_idx).unwrap();
                    println!("{} {} restarted", cmd.prefix(), full_command);
                }
            }

            Event::Output { command_idx, line } => {
                let cmd = config.commands.get(command_idx).unwrap();
                println!("{} {}", cmd.prefix(), line);
            }

            Event::Exit {
                command_idx,
                status,
            } => {
                let cmd = config.commands.get(command_idx).unwrap();
                let full_command = config.args.commands.get(command_idx).unwrap();
                println!("{} {} exited with {}", cmd.prefix(), full_command, status);

                // -1 because `fetch_sub` returns the state _before_ the subtraction operation
                let num_processes = processes_alive.fetch_sub(1, Ordering::Relaxed) - 1;
                trace!("Alive processes: {}", num_processes);

                // We're mirroring the behaviour of concurrently, where restarts only happen if
                // the process exited with a non-success code. This seems to make sense, but maybe
                // there is a case for an option to always restart?
                if !status.success()
                    && (cmd.restart_indefinitely
                        || cmd.restart_tries.fetch_sub(1, Ordering::Relaxed) > 0)
                {
                    let tx = tx_orig.clone();
                    processes.spawn(async move {
                        if !config.restart_after.is_zero() {
                            tokio::time::sleep(config.restart_after).await;
                        }

                        tx.send(Event::Spawn {
                            command_idx,
                            is_restart: true,
                        })
                        .await
                        .context("Failed to send spawn message")
                    });
                } else if num_processes < 1 {
                    debug!("No more processes. Stopping main loop.");
                    break;
                }
            }
        }
    }

    // Why need to drop the sending end of this channel, so that the receiving end will
    // close once all messages have been delivered. If we don't drop this end here, the
    // draining loop below will wait indefinitely.
    drop(tx_orig);

    while let Some(event) = rx.recv().await {
        match event {
            Event::Output { command_idx, line } => {
                let cmd = config.commands.get(command_idx).unwrap();
                println!("{} {}", cmd.prefix(), line);
            }
            x => {
                error!("{:?}", x)
            }
        }
    }

    while let Some(res) = processes.join_next().await {
        if let Err(err) = flatten_errors(res) {
            debug!("Spawned task failed with error: {:?}", err);
        }
    }

    Ok(())
}

fn flatten_errors<T, E1, E2>(res: Result<Result<T, E1>, E2>) -> Result<T>
where
    E1: Into<anyhow::Error>,
    E2: Into<anyhow::Error>,
{
    match res {
        Ok(Ok(x)) => Ok(x),
        Ok(Err(e)) => Err(e.into()),
        Err(e) => Err(e.into()),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let args: Args = Args::parse();
    let config: &'static Config = Box::leak(Box::new(args.try_into()?));
    debug!("{:#?}", config);

    if config.commands.len() < 1 {
        Args::command().print_long_help()?;
        println!();
        bail!("No commands were given");
    }

    // This is the channel that is used to communicate everything that's happening
    // in the spawned processes back here, we're output is handled.
    let (tx, rx) = mpsc::channel::<Event>(OUTPUT_CHANNEL_BUFFER_SIZE);

    event_loop(config, tx, rx).await
}
