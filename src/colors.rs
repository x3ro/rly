use anyhow::{bail, Result};
use colored::*;

pub fn colorize(format: &str, input: &str) -> Result<String> {
    let mut result = input.clear();
    for modifier in format.split('.') {
        result = match modifier {
            "reset" => result.clear(),
            "bold" => result.bold(),
            "dim" => result.dimmed(),
            "italic" => result.italic(),
            "underline" => result.underline(),
            "hidden" => result.hidden(),
            "strikethrough" => result.strikethrough(),
            // "overline" => todo!(),
            // "inverse" => todo!(),
            // "visible" => todo!(),
            "black" => result.black(),
            "red" => result.red(),
            "green" => result.green(),
            "yellow" => result.yellow(),
            "blue" => result.blue(),
            "magenta" => result.magenta(),
            "cyan" => result.cyan(),
            "white" => result.white(),
            "blackBright" | "gray" | "grey" => result.bright_black(),
            "redBright" => result.bright_red(),
            "greenBright" => result.bright_green(),
            "yellowBright" => result.bright_yellow(),
            "blueBright" => result.bright_blue(),
            "magentaBright" => result.bright_magenta(),
            "cyanBright" => result.bright_cyan(),
            "whiteBright" => result.bright_white(),

            "bgBlack" => result.on_black(),
            "bgRed" => result.on_red(),
            "bgGreen" => result.on_green(),
            "bgYellow" => result.on_yellow(),
            "bgBlue" => result.on_blue(),
            "bgMagenta" => result.on_magenta(),
            "bgCyan" => result.on_cyan(),
            "bgWhite" => result.on_white(),
            "bgBlackBright" | "bgGray" | "bgGrey" => result.on_bright_black(),
            "bgRedBright" => result.on_bright_red(),
            "bgGreenBright" => result.on_bright_green(),
            "bgYellowBright" => result.on_bright_yellow(),
            "bgBlueBright" => result.on_bright_blue(),
            "bgMagentaBright" => result.on_bright_magenta(),
            "bgCyanBright" => result.on_bright_cyan(),
            "bgWhiteBright" => result.on_bright_white(),

            x => bail!("Unknown/unsupported color modifier '{}'", x),
        };
    }
    Ok(result.to_string())
}
