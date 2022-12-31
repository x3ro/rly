use crate::{Args, Command, Commands};

#[derive(Debug)]
pub struct Config {
    pub args: Args,
    pub commands: Vec<Command>,
    pub names: Vec<String>,
}

impl TryFrom<Args> for Config {
    type Error = anyhow::Error;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let mut names: Vec<_> = match &args.names {
            None => (0..args.commands.len()).map(|i| i.to_string()).collect(),
            Some(s) => s
                .split(args.name_separator)
                .map(|s| s.to_string())
                .collect(),
        };

        while args.commands.len() > names.len() {
            let last = names.last().unwrap();
            names.push(last.clone());
        }

        let mut config = Config {
            args,
            commands: vec![],
            names,
        };

        config.commands = Commands::from(&config)?;
        Ok(config)
    }
}
