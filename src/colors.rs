use anyhow::{bail, Result};
use colored::*;

#[rustfmt::skip]
const AUTO_COLORS: &[&str] = &[
    "red",  "green", "yellow", "blue", "magenta", "cyan", "white", "gray",
    "redBright",  "greenBright", "yellowBright", "blueBright", "magentaBright", "cyanBright"
];

fn auto_color(idx: usize, input: ColoredString) -> Result<ColoredString> {
    let color = AUTO_COLORS[idx % AUTO_COLORS.len()];
    apply_modifier_by_name(input, color)
}

fn apply_modifier_by_name(input: ColoredString, modifier: &str) -> Result<ColoredString> {
    let result = match modifier {
        "reset" => input.clear(),
        "bold" => input.bold(),
        "dim" => input.dimmed(),
        "italic" => input.italic(),
        "underline" => input.underline(),
        "hidden" => input.hidden(),
        "strikethrough" => input.strikethrough(),
        // "overline" => todo!(),
        // "inverse" => todo!(),
        // "visible" => todo!(),
        "black" => input.black(),
        "red" => input.red(),
        "green" => input.green(),
        "yellow" => input.yellow(),
        "blue" => input.blue(),
        "magenta" => input.magenta(),
        "cyan" => input.cyan(),
        "white" => input.white(),
        "blackBright" | "gray" | "grey" => input.bright_black(),
        "redBright" => input.bright_red(),
        "greenBright" => input.bright_green(),
        "yellowBright" => input.bright_yellow(),
        "blueBright" => input.bright_blue(),
        "magentaBright" => input.bright_magenta(),
        "cyanBright" => input.bright_cyan(),
        "whiteBright" => input.bright_white(),

        "bgBlack" => input.on_black(),
        "bgRed" => input.on_red(),
        "bgGreen" => input.on_green(),
        "bgYellow" => input.on_yellow(),
        "bgBlue" => input.on_blue(),
        "bgMagenta" => input.on_magenta(),
        "bgCyan" => input.on_cyan(),
        "bgWhite" => input.on_white(),
        "bgBlackBright" | "bgGray" | "bgGrey" => input.on_bright_black(),
        "bgRedBright" => input.on_bright_red(),
        "bgGreenBright" => input.on_bright_green(),
        "bgYellowBright" => input.on_bright_yellow(),
        "bgBlueBright" => input.on_bright_blue(),
        "bgMagentaBright" => input.on_bright_magenta(),
        "bgCyanBright" => input.on_bright_cyan(),
        "bgWhiteBright" => input.on_bright_white(),

        x => bail!("Unknown/unsupported color modifier '{}'", x),
    };

    Ok(result)
}

/// Apply terminal colors to the given `input`, based on the `format`
/// (as documented in [`crate::cli::Args::prefix_colors`]
pub fn colorize(command_idx: usize, format: &str, input: &str) -> Result<String> {
    let mut result = input.clear();
    for modifier in format.split('.') {
        if modifier == "auto" {
            result = auto_color(command_idx, result)?;
        } else {
            result = apply_modifier_by_name(result, modifier)?;
        }
    }
    Ok(result.to_string())
}
