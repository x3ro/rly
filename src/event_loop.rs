use std::future::Future;
use std::process::ExitStatus;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use log::{debug, error, trace};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::signal;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinSet;

use crate::{Command, Config};

macro_rules! rly_println {
    ($cmd:expr, $($arg:tt)*) => {{
        if !$cmd.disable_output() {
            println!($($arg)*);
        }
    }};
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

struct State {
    config: &'static Config,
    children_alive: AtomicUsize,
    task_set: JoinSet<Result<()>>,
    kill_channels: Vec<Option<oneshot::Sender<()>>>,
    tx: mpsc::Sender<Event>,
}

impl State {
    pub fn shut_down(self) -> JoinSet<Result<()>> {
        self.task_set
    }
}

async fn handle_ctrlc() -> Result<()> {
    match signal::ctrl_c().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Unable to listen to Ctrl-C: {}", e);
            Err(anyhow!(e))
        }
    }
}

fn should_kill_others(state: &State, status: &ExitStatus) -> bool {
    // If the kill channels are empty, that means that we've already
    // sent kill signals to the processes. In that case, we shouldn't
    // try to do it again.
    if state.kill_channels.is_empty() {
        return false;
    }

    if state.config.kill_others_on_fail {
        return !status.success();
    }

    if !state.config.kill_others {
        return false;
    }

    true
}

async fn handle_next_event(
    config: &'static Config,
    state: &mut State,
    rx: &mut mpsc::Receiver<Event>,
) -> Result<bool> {
    match rx.recv().await {
        Some(Event::Spawn {
            command_idx,
            is_restart,
        }) => {
            handle_spawn_event(state, command_idx, is_restart).await?;
            Ok(true)
        }

        Some(Event::Output { command_idx, line }) => {
            let cmd = config.commands.get(command_idx).unwrap();
            rly_println!(cmd, "{} {}", cmd.prefix(), line);
            Ok(true)
        }

        Some(Event::Exit {
            command_idx,
            status,
        }) => {
            let cmd = config.commands.get(command_idx).unwrap();
            let full_command = &config.commands.get(command_idx).unwrap().command;
            rly_println!(
                cmd,
                "{} {} exited with {}",
                cmd.prefix(),
                full_command,
                status
            );

            // -1 because `fetch_sub` returns the state _before_ the subtraction operation
            let num_processes = state.children_alive.fetch_sub(1, Ordering::Relaxed) - 1;
            debug!("{cmd} exited. Alive processes now: {}", num_processes);

            // We're mirroring the behaviour of concurrently, where restarts only happen if
            // the process exited with a non-success code. This seems to make sense, but maybe
            // there is a case for an option to always restart?
            if !status.success()
                && (cmd.restart_indefinitely
                    || cmd.restart_tries.fetch_sub(1, Ordering::Relaxed) > 0)
            {
                let tx = state.tx.clone();
                state.task_set.spawn(async move {
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

                Ok(true)
            } else if should_kill_others(state, &status) {
                rly_println!(cmd, "--> Sending SIGTERM to other processes..");
                for mut opt in state.kill_channels.drain(..) {
                    if let Some(tx) = opt.take() {
                        tx.send(()).unwrap_or(());
                    }
                }

                Ok(true)
            } else if num_processes < 1 {
                debug!("No more processes. Stopping main loop.");
                Ok(false)
            } else {
                Ok(true)
            }
        }

        None => Ok(false),
    }
}

pub async fn event_loop(config: &'static Config) -> Result<()> {
    // This is the channel that is used to communicate everything that's happening
    // in the spawned processes back here, where output is handled.
    let (tx, mut rx) = mpsc::channel::<Event>(OUTPUT_CHANNEL_BUFFER_SIZE);

    let mut state = State {
        config,
        children_alive: AtomicUsize::new(0),
        task_set: JoinSet::new(),
        kill_channels: vec![],
        tx,
    };

    for _ in 0..config.commands.len() {
        state.kill_channels.push(None);
    }

    for (command_idx, _) in config.commands.iter().enumerate() {
        state
            .tx
            .send(Event::Spawn {
                command_idx,
                is_restart: false,
            })
            .await
            .unwrap()
    }

    loop {
        tokio::select! {
            _ = handle_ctrlc() => {
                rly_println!(config, "Ctrl-C issued");
                if state.kill_channels.is_empty() {
                  break;
                } else {
                    rly_println!(config, "Terminating all processes..");
                    for mut opt in state.kill_channels.drain(..) {
                        if let Some(tx) = opt.take() {
                            tx.send(()).unwrap_or(());
                        }
                    }
                }
            },
            result = handle_next_event(config, &mut state, &mut rx) => {
                if !result? {
                  break;
                }
            },
        }
    }

    trace!("Main event loop has stopped.");

    // We need to drop the sending end of this channel, so that the receiving end will
    // close once all messages have been delivered. If we don't drop this end here, the
    // draining loop below will wait indefinitely.
    let mut task_set = state.shut_down();

    while let Some(event) = rx.recv().await {
        match event {
            Event::Output { command_idx, line } => {
                let cmd = config.commands.get(command_idx).unwrap();
                rly_println!(cmd, "{} {}", cmd.prefix(), line);
            }
            x => {
                error!("{:?}", x)
            }
        }
    }

    while let Some(res) = task_set.join_next().await {
        if let Err(err) = flatten_errors(res) {
            debug!("Spawned task failed with error: {:?}", err);
        }
    }

    Ok(())
}

fn output_listener<R: AsyncRead + Unpin>(
    name: &'static str,
    command_idx: usize,
    cmd: &'static Command,
    reader: R,
    tx: mpsc::Sender<Event>,
) -> impl Future<Output = Result<()>> {
    async move {
        trace!("{name} reader task for {cmd} started");

        let mut reader = BufReader::new(reader).lines();
        while let Some(line) = reader.next_line().await? {
            tx.send(Event::Output { line, command_idx }).await?
        }

        trace!("{name} reader task for {cmd} stopped");
        Ok(())
    }
}

async fn handle_spawn_event(state: &mut State, command_idx: usize, is_restart: bool) -> Result<()> {
    let cmd = state.config.commands.get(command_idx).unwrap();
    let mut child = cmd.tokio_command().spawn().expect("Failed to spawn child");

    let pid = child
        .id()
        .expect("Successfully spawned child should have a PID");
    cmd.pid.store(pid, Ordering::Relaxed);
    debug!("Spawned command {cmd}");

    if state.config.timings {
        rly_println!(cmd, "{}", cmd.status_msg("started"));
    }

    if !state.config.raw {
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to acquire stdout handle"))?;

        state.task_set.spawn(output_listener(
            "stdout",
            command_idx,
            cmd,
            stdout,
            state.tx.clone(),
        ));

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("Failed to acquire stderr handle"))?;

        state.task_set.spawn(output_listener(
            "stderr",
            command_idx,
            cmd,
            stderr,
            state.tx.clone(),
        ));
    }

    // This is the task that waits for the child's exit status
    let (kill_tx, kill_rx) = oneshot::channel::<()>();
    state.kill_channels[command_idx] = Some(kill_tx);
    let tx = state.tx.clone();
    state.task_set.spawn(async move {
        tokio::select! {
            status = child.wait() => {
                let status = status?;
                trace!("Task with pid {pid} exited with {status}");

                tx.send(Event::Exit {
                    command_idx,
                    status,
                })
                .await?;
            }

            _ = kill_rx => {
                trace!("Received kill signal for {cmd}");

                if cfg!(target_os = "windows") {
                  if let Err(e) = child.start_kill() {
                    eprintln!("Could not kill process: {:?}", e);
                  };
                } else {
                  // On Unix, it is preferable to send the SIGTERM signal as it allows
                  // the process to shut down gracefully
                  let pid = nix::unistd::Pid::from_raw(child.id().unwrap() as i32);
                  nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGTERM).expect("Failed to send SIGTERM");
                }

                let status =
                  match tokio::time::timeout(Duration::from_secs(2), child.wait()).await {
                    Ok(Ok(status)) => {
                      status
                    },
                    Ok(Err(e)) => {
                      eprintln!("Could not wait for process");
                      Err(e)?
                    }
                    Err(_) => {
                      eprintln!("Process did not terminate within 2s, sending SIGKILL...");
                      let pid = nix::unistd::Pid::from_raw(child.id().unwrap() as i32);
                      nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGKILL).expect("Failed to send SIGKILL");
                      ExitStatus::default()
                    },
                  };

                debug!("{cmd} killed with {status}");
                tx.send(Event::Exit {
                    command_idx,
                    status,
                }).await?;
            }
        }

        Ok(())
    });

    state.children_alive.fetch_add(1, Ordering::SeqCst);
    if is_restart {
        rly_println!(cmd, "{} {} restarted", cmd.prefix(), cmd.command);
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
