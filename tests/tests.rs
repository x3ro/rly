use pretty_assertions::assert_str_eq;

use crate::util::setup;

#[macro_use]
mod util;

#[test]
fn it_runs_a_basic_command() {
    let (dir, mut cmd) = setup("it_runs_a_basic_command");
    dir.create("some-file", "some-file-contents");
    dir.create("some-other-file", "some-other-file-contents");

    let out = cmd
        .arg("ls .")
        .arg("sleep 0.1; cat some-file; exit 1")
        .stdout();

    let expected = r#"[0] some-file
[0] some-other-file
[0] ls . exited with code 0
[1] some-file-contents
[1] sleep 0.1; cat some-file; exit 1 exited with code 1
"#;

    assert_str_eq!(expected, out);
}

#[test]
fn it_supports_names() {
    let (dir, mut cmd) = setup("it_supports_names");
    dir.create("some-file", "some-file-contents");
    dir.create("some-other-file", "some-other-file-contents");

    let out = cmd
        .args(&["--names", "ls,cat"])
        .arg("ls .")
        .arg("sleep 0.1; cat some-file; exit 1")
        .stdout();

    let expected = r#"[ls] some-file
[ls] some-other-file
[ls] ls . exited with code 0
[cat] some-file-contents
[cat] sleep 0.1; cat some-file; exit 1 exited with code 1
"#;

    assert_str_eq!(expected, out);
}

#[test]
fn it_repeats_last_name_if_not_enough_names_are_given() {
    let (dir, mut cmd) = setup("it_repeats_last_name_if_not_enough_names_are_given");
    dir.create("some-file", "some-file-contents");
    dir.create("some-other-file", "some-other-file-contents");

    let out = cmd
        .args(&["--names", "ls,repeat"])
        .arg("ls .")
        .arg("sleep 0.2; cat some-file; exit 1")
        .arg("sleep 0.1; cat some-other-file")
        .stdout();

    let expected = r#"[ls] some-file
[ls] some-other-file
[ls] ls . exited with code 0
[repeat] some-other-file-contents
[repeat] sleep 0.1; cat some-other-file exited with code 0
[repeat] some-file-contents
[repeat] sleep 0.2; cat some-file; exit 1 exited with code 1
"#;

    assert_str_eq!(expected, out);
}

#[test]
fn it_supports_custom_prefixes() {
    let (dir, mut cmd) = setup("it_supports_custom_prefixes");
    dir.create("some-file", "some-file-contents");

    let out = cmd
        .arg("cat some-file")
        .args(&["--prefix", "[{index}-{command}]"])
        .arg("--prefix-length=14")
        .stdout();

    let expected = r#"[0-cat so..e-file] some-file-contents
[0-cat so..e-file] cat some-file exited with code 0
"#;

    assert_str_eq!(expected, out);
}

#[test]
fn it_supports_time_prefix() {
    let (dir, mut cmd) = setup("it_supports_time_prefix");
    dir.create("some-file", "some-file-contents");

    let timestamp_format = "%Y-%m-%d %H:%M";
    let out = cmd
        .arg("cat some-file")
        .args(&["--prefix", "[{time}]"])
        .args(&["--timestamp-format", timestamp_format])
        .arg("--prefix-length=25")
        .stdout();

    let expected_time = chrono::Local::now().format(timestamp_format);
    let expected = format!(
        r#"[{0}] some-file-contents
[{0}] cat some-file exited with code 0
"#,
        expected_time
    );

    assert_str_eq!(expected, out);
}

#[test]
fn it_supports_disabling_color() {
    let (dir, mut cmd) = setup("it_supports_disabling_color");
    dir.create("some-file", "some-file-contents");

    let out = cmd
        .arg("cat some-file")
        .args(&["--prefix", "[{index}]"])
        .args(&["--prefix-colors", "blue.bgRed"])
        .arg("--no-color")
        .stdout();

    let expected = format!(
        r#"[0] some-file-contents
[0] cat some-file exited with code 0
"#
    );

    assert_str_eq!(expected, out);
}

#[test]
fn it_supports_colors() {
    let (dir, mut cmd) = setup("it_supports_colors");
    dir.create("some-file", "some-file-contents");

    let out = cmd
        .arg("cat some-file")
        .arg("sleep 0.1; echo foo")
        .arg("sleep 0.2; echo bar")
        .args(&["--prefix-colors", "blue.strikethrough.bgRed,green"])
        .stdout();

    let expected_prefix = "\u{1b}[9;41;34m[0]\u{1b}[0m";
    let expected_prefix_green_1 = "\u{1b}[32m[1]\u{1b}[0m";
    let expected_prefix_green_2 = "\u{1b}[32m[2]\u{1b}[0m";
    let expected = format!(
        "{0} some-file-contents
{0} cat some-file exited with code 0
{1} foo
{1} sleep 0.1; echo foo exited with code 0
{2} bar
{2} sleep 0.2; echo bar exited with code 0
",
        expected_prefix, expected_prefix_green_1, expected_prefix_green_2
    );

    assert_str_eq!(escape_debug_by_line(expected), escape_debug_by_line(out));
}

fn escape_debug_by_line(s: impl AsRef<str>) -> String {
    s.as_ref()
        .escape_debug()
        .to_string()
        .split("\\n")
        .collect::<Vec<_>>()
        .join("\n")
}
