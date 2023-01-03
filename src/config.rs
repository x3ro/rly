use crate::{Args, Command, Commands};

#[derive(Debug)]
pub struct Config {
    pub args: Args,
    pub commands: Vec<Command>,
    pub names: Vec<String>,
    pub prefix_colors: Vec<String>,
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

        let mut config = Config {
            args,
            commands: vec![],
            names,
            prefix_colors,
        };

        config.commands = Commands::from(&config)?;
        Ok(config)
    }
}
