use std::process::Stdio;

use anyhow::{Context, Result};

use crate::config::Config;

pub struct Commands;

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub command: String,
}

impl From<&Command> for tokio::process::Command {
    fn from(cmd: &Command) -> Self {
        let mut res = tokio::process::Command::new("sh");
        res.arg("-c").arg(&cmd.command).stdout(Stdio::piped());
        res
    }
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
        let name = config.args.prefix
            .replace("{index}", &format!("{}", idx))
            .replace("{command}", cmd.as_ref())
            .replace("{name}", config.names.get(idx).unwrap());

        let command = Command {
            name,
            command: cmd.as_ref().to_string(),
        };

        Ok(command)
    }
}
