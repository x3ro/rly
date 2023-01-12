use std::time::Duration;

use crate::{Args, Command, Commands};

#[derive(Debug)]
pub struct Config {
    pub commands: Vec<Command>,
    pub names: Vec<String>,
    pub prefix_colors: Vec<String>,
    pub restart_after: Duration,

    pub prefix: String,
    pub raw: bool,
    pub prefix_length: usize,
    pub no_color: bool,
    pub timestamp_format: String,
    pub restart_tries: i32,
    pub kill_others: bool,
    pub kill_others_on_fail: bool,
}

fn maybe_repeat(input: &str, separator: char, count: usize) -> Vec<String> {
    let mut result: Vec<_> = input.split(separator).map(|s| s.to_string()).collect();

    let last = result.last().unwrap(/* Guaranteed to have at least one element */).clone();
    while count > result.len() {
        result.push(last.clone());
    }

    result
}

impl TryFrom<Args> for Config {
    type Error = anyhow::Error;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let names: Vec<_> = match &args.names {
            None => (0..args.commands.len()).map(|i| i.to_string()).collect(),
            Some(s) => maybe_repeat(s, args.name_separator, args.commands.len()),
        };

        let prefix_colors: Vec<_> = maybe_repeat(&args.prefix_colors, ',', args.commands.len());
        let restart_after = Duration::from_millis(args.restart_after);

        let mut config = Config {
            commands: vec![],
            names,
            prefix_colors,
            restart_after,
            prefix: args.prefix,
            raw: args.raw,
            prefix_length: args.prefix_length,
            no_color: args.no_color,
            timestamp_format: args.timestamp_format,
            restart_tries: args.restart_tries,
            kill_others: args.kill_others,
            kill_others_on_fail: args.kill_others_on_fail,
        };

        config.commands = Commands::from(&config, args.commands.as_slice())?;
        Ok(config)
    }
}
