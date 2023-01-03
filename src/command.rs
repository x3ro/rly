use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::process::Child;
use tokio::sync::Mutex;

use crate::colors::colorize;
use crate::config::Config;

pub struct Commands;

#[derive(Debug)]
pub struct Command {
    prefix: String,
    timestamp_format: String,

    pub command: String,
    // TODO: It's a bit difficult to
    //       get the PID into the Command::name, and for now I'm
    //       spawning the process when creating the command.
    //       Not super happy with this, maybe there's a better way?
    pub child: Arc<Mutex<Option<Child>>>,
}

impl Command {
    pub fn prefix(&self) -> String {
        self.prefix.replace(
            "{time}",
            &chrono::prelude::Local::now()
                .format(&self.timestamp_format)
                .to_string(),
        )
    }

    fn shorten(prefix_length: usize, name: &str) -> String {
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
    pub fn from(config: &Config) -> Result<Vec<Command>> {
        let args = &config.args;
        let commands: Vec<Command> = args
            .commands
            .iter()
            .enumerate()
            .map(|(idx, cmd)| Self::prepare_command(config, idx, cmd))
            .collect::<Result<Vec<_>>>()
            .with_context(|| format!("Failed to parse given commands: {:?}", args.commands))?;

        Ok(commands)
    }

    fn prepare_command(config: &Config, idx: usize, cmd: impl AsRef<str>) -> Result<Command> {
        let mut runnable = tokio::process::Command::new("sh");
        runnable.arg("-c").arg(cmd.as_ref()).stdout(Stdio::piped());

        let child = runnable.spawn()?;
        let pid = child
            .id()
            .expect("Successfully spawned child should have a PID");

        let mut prefix = config
            .args
            .prefix
            .replace("{index}", &format!("{}", idx))
            .replace("{pid}", &format!("{}", pid))
            .replace(
                "{command}",
                &Command::shorten(config.args.prefix_length, cmd.as_ref()),
            )
            .replace("{name}", config.names.get(idx).unwrap());

        if !config.args.no_color {
            prefix = colorize(config.prefix_colors.get(idx).unwrap(), &prefix)?;
        }

        let command = Command {
            prefix,
            timestamp_format: config.args.timestamp_format.clone(),
            command: cmd.as_ref().to_string(),
            child: Arc::new(Mutex::new(Some(child))),
        };

        Ok(command)
    }
}
