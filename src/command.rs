use std::fmt::Formatter;
use std::process::Stdio;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use anyhow::{Context, Result};
use tokio::process::Command as TokioCommand;

use crate::colors::colorize;
use crate::config::Config;

pub struct Commands;

#[derive(Debug)]
pub struct Command {
    prefix: String,
    timestamp_format: String,
    pub raw: bool,
    pub hide: bool,

    pub command: String,
    pub pid: AtomicU32,
    pub restart_indefinitely: bool,
    pub restart_tries: AtomicI32,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "'{}' (pid: {})",
            self.command,
            self.pid.load(Ordering::Relaxed)
        ))
    }
}

impl Command {
    pub fn prefix(&self) -> String {
        self.prefix
            .replace(
                "{time}",
                &chrono::prelude::Local::now()
                    .format(&self.timestamp_format)
                    .to_string(),
            )
            .replace("{pid}", &self.pid.load(Ordering::Relaxed).to_string())
    }

    pub fn tokio_command(&self) -> TokioCommand {
        let mut runnable = tokio::process::Command::new("sh");
        runnable.arg("-c").arg(&self.command);

        if !self.raw {
            runnable.stdout(Stdio::piped()).stderr(Stdio::piped());
        }

        runnable
    }

    fn shorten(prefix_length: usize, name: &str) -> String {
        if name.len() <= prefix_length {
            return name.to_string();
        }

        // -1 because of the two-character ellipsis (..), one
        // character for each part.
        const ELLIPSIS: &str = "..";
        const ELLIPSIS_LENGTH: usize = {
            if ELLIPSIS.len() % 2 != 0 {
                panic!("Ellipsis length must be even");
            }
            ELLIPSIS.len()
        };

        let part_length = (prefix_length / 2) - (ELLIPSIS_LENGTH / 2);
        let mut left_length = part_length;
        let right_length = part_length;

        // If we're coming up short it's because prefix_length is uneven. In that case,
        // since we're dividing by two, we can only be short one character. We add this
        // character back to the left-hand side of the split.
        if (left_length + right_length + ELLIPSIS_LENGTH) < prefix_length {
            left_length += 1;
        }

        let left = &name[0..left_length];
        let right = &name[(name.len() - right_length)..];

        format!("{}{}{}", left, ELLIPSIS, right)
    }
}

impl Commands {
    pub fn from(config: &Config, commands: &[String]) -> Result<Vec<Command>> {
        let commands: Vec<Command> = commands
            .iter()
            .enumerate()
            .map(|(idx, cmd)| Self::prepare_command(config, idx, cmd))
            .collect::<Result<Vec<_>>>()
            .with_context(|| format!("Failed to parse given commands: {:?}", commands))?;

        Ok(commands)
    }

    fn prepare_command(config: &Config, idx: usize, cmd: impl AsRef<str>) -> Result<Command> {
        let name = config.names.get(idx).unwrap();
        let idx_str = idx.to_string();
        let hide = config.hide.contains(name) || config.hide.contains(&idx_str);

        let mut prefix = config
            .prefix
            .replace("{index}", &format!("{}", idx))
            .replace(
                "{command}",
                &Command::shorten(config.prefix_length, cmd.as_ref()),
            )
            .replace("{name}", name);

        if !config.no_color {
            prefix = colorize(idx, config.prefix_colors.get(idx).unwrap(), &prefix)?;
        }

        let command = Command {
            prefix,
            hide,
            raw: config.raw,
            timestamp_format: config.timestamp_format.clone(),
            command: cmd.as_ref().to_string(),
            pid: Default::default(),
            restart_tries: AtomicI32::new(config.restart_tries),
            restart_indefinitely: config.restart_tries < 0,
        };

        Ok(command)
    }
}
