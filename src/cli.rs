use clap::Parser;

// Descriptions of the arguments are mostly verbatim-copied from
// the `concurrently` project, which is MIT licensed and
// Copyright (c) by Kimmo Brunfeldt (and possible contributors)

// #[derive(Foo, Debug, Default, PartialEq)]
// #[clap(next_help_heading = "HEADING A")]
// struct Foobar {
//     /// WAT
//     #[clap(short, long)]
//     wobbel: String,
// }

#[derive(Parser, Debug, Default, PartialEq)]
#[clap(about, version, author)]
pub struct Args {
    #[clap(global = true)]
    pub commands: Vec<String>,

    /// List of custom names to be used in prefix template.
    ///
    /// Example names: "main,browser,server"
    #[clap(short, long)]
    pub names: Option<String>,

    /// The character to split <names> on.
    ///
    /// Example usage:
    /// -n "styles|scripts|server" --name-separator "|"
    #[clap(long, default_value = ",")]
    pub name_separator: char,

    /// Output only raw output of processes, disables
    /// prettifying and concurrently coloring.
    // #[clap(short, long)]
    // pub raw: bool,

    /// Disables colors from logging.
    #[clap(long, default_value = "false")]
    pub no_color: bool,

    /// Comma-separated list of processes for which to
    /// hide the output. The processes can be identified
    /// by their name or index.
    // #[clap(long)]
    // pub hide: Option<String>,

    /// Order the output as if the commands were run sequentially.
    // #[clap(short, long)]
    // group: bool,

    /// Show timing information for all processes.
    // #[clap(long)]
    // pub timings: bool,

    /// Passthrough additional arguments to commands
    /// (accessible via placeholders) instead of treating
    /// them as commands.
    // #[clap(short = 'P', long)]
    // pub passthrough_arguments: bool,

    /// Prefix used in logging for each process.
    /// Possible values: index, pid, time, command, name,
    /// none, or a template. Example template: "{time}-{pid}"
    #[clap(short, long, default_value = "[{name}]")]
    pub prefix: String,

    /// Comma-separated list of chalk colors to use on
    /// prefixes. If there are more commands than colors, the
    /// last color will be repeated.
    ///
    /// - Available modifiers: reset, bold, dim, italic,
    ///   underline, inverse, hidden, strikethrough
    ///
    /// - Available colors: black, red, green, yellow, blue,
    ///   magenta, cyan, white, gray or auto for
    ///   an automatically picked color
    ///
    /// - Available background colors: bgBlack, bgRed,
    ///   bgGreen, bgYellow, bgBlue, bgMagenta, bgCyan, bgWhite
    ///
    /// See https://www.npmjs.com/package/chalk for more
    /// information.
    #[clap(short = 'c', long, default_value = "reset")]
    pub prefix_colors: String,

    /// Limit how many characters of the command is displayed
    /// in prefix. The option can be used to shorten the
    /// prefix when it is set to "command"
    #[clap(short = 'l', long, default_value = "10")]
    pub prefix_length: usize,

    /// Specify the timestamp in chrono::format syntax.
    #[clap(short, long, default_value = "%Y-%m-%d %H:%M:%S.%3f")]
    pub timestamp_format: String,

    /// Kill other processes if one exits or dies.
    #[clap(short, long)]
    pub kill_others: bool,

    /// Kill other processes if one exits with non zero
    /// status code.
    #[clap(long)]
    pub kill_others_on_fail: bool,

    /// How many times a process that died should restart.
    /// Negative numbers will make the process restart forever.
    #[clap(long, default_value = "0")]
    pub restart_tries: i32,

    /// Delay time to respawn the process, in milliseconds.
    #[clap(long, default_value = "0")]
    pub restart_after: u64,
    // #[clap(flatten)]
    // wat: Foobar,
}

#[cfg(test)]
mod tests {
    use super::*;

    const CMD: [&str; 1] = ["rly"];

    fn try_parse(args: &[&str]) -> Result<Args, clap::Error> {
        let iter = CMD.iter().chain(args);
        Args::try_parse_from(iter)
    }

    #[test]
    fn test_no_color() {
        let res = try_parse(&["--no-color", "npm run watch-less", "npm run watch-js"]).unwrap();

        let commands = vec![
            "npm run watch-less".to_string(),
            "npm run watch-js".to_string(),
        ];

        assert_eq!(commands, res.commands);
        assert!(res.no_color);
    }
}
