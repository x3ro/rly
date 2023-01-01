use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::process::Child;
use tokio::sync::Mutex;

use crate::config::Config;

pub struct Commands;

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub command: String,
    // TODO: It's a bit difficult to
    //       get the PID into the Command::name, and for now I'm
    //       spawning the process when creating the command.
    //       Not super happy with this, maybe there's a better way?
    pub child: Arc<Mutex<Option<Child>>>,
}

impl Commands {
    pub fn from(config: &Config) -> Result<Vec<Command>> {
        let args = &config.args;
        let commands: Vec<Command> = args
            .commands
            .iter()
            .enumerate()
            .map(|(idx, cmd)| Self::parse_command(config, idx, cmd))
            .collect::<Result<Vec<_>>>()
            .with_context(|| format!("Failed to parse given commands: {:?}", args.commands))?;

        Ok(commands)
    }

    fn parse_command(config: &Config, idx: usize, cmd: impl AsRef<str>) -> Result<Command> {
        let mut runnable = tokio::process::Command::new("sh");
        runnable.arg("-c").arg(cmd.as_ref()).stdout(Stdio::piped());

        let child = runnable.spawn()?;
        let pid = child
            .id()
            .expect("Successfully spawned child should have a PID");

        let name = config
            .args
            .prefix
            .replace("{index}", &format!("{}", idx))
            .replace("{pid}", &format!("{}", pid))
            .replace("{command}", cmd.as_ref())
            .replace("{name}", config.names.get(idx).unwrap());

        let command = Command {
            name,
            command: cmd.as_ref().to_string(),
            child: Arc::new(Mutex::new(Some(child))),
        };

        Ok(command)
    }
}
