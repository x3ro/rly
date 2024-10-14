use crate::util::{assert_eq_lines_unordered, setup};

#[macro_use]
mod util;

#[test]
fn it_prints_help_when_no_commands_are_given() {
    let (_, mut cmd) = setup("it_prints_help_when_no_commands_are_given()");
    let out = cmd.err_stdout();
    assert!(out.contains("Options:"))
}

#[test]
fn it_runs_a_single_basic_command() {
    let (dir, mut cmd) = setup("it_runs_a_single_basic_command");
    dir.create("some-file", "some-file-contents");

    let out = cmd.arg("ls").stdout();

    let expected = r#"[0] some-file
[0] ls exited with exit status: 0
"#;

    assert_eq_lines_unordered(expected, out);
}

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
[0] ls . exited with exit status: 0
[1] some-file-contents
[1] sleep 0.1; cat some-file; exit 1 exited with exit status: 1
"#;

    assert_eq_lines_unordered(expected, out);
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
[ls] ls . exited with exit status: 0
[cat] some-file-contents
[cat] sleep 0.1; cat some-file; exit 1 exited with exit status: 1
"#;

    assert_eq_lines_unordered(expected, out);
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
[ls] ls . exited with exit status: 0
[repeat] some-other-file-contents
[repeat] sleep 0.1; cat some-other-file exited with exit status: 0
[repeat] some-file-contents
[repeat] sleep 0.2; cat some-file; exit 1 exited with exit status: 1
"#;

    assert_eq_lines_unordered(expected, out);
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

    let expected = r#"[0-cat some-file] some-file-contents
[0-cat some-file] cat some-file exited with exit status: 0
"#;

    assert_eq_lines_unordered(expected, out);
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
[{0}] cat some-file exited with exit status: 0
"#,
        expected_time
    );

    assert_eq_lines_unordered(expected, out);
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
[0] cat some-file exited with exit status: 0
"#
    );

    assert_eq_lines_unordered(expected, out);
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
{0} cat some-file exited with exit status: 0
{1} foo
{1} sleep 0.1; echo foo exited with exit status: 0
{2} bar
{2} sleep 0.2; echo bar exited with exit status: 0
",
        expected_prefix, expected_prefix_green_1, expected_prefix_green_2
    );

    assert_eq_lines_unordered(escape_debug_by_line(expected), escape_debug_by_line(out));
}

#[test]
fn it_supports_auto_colors() {
    let (dir, mut cmd) = setup("it_supports_auto_colors");
    dir.create("some-file", "some-file-contents");

    let out = cmd
        .arg("cat some-file")
        .arg("sleep 0.1; echo foo")
        .arg("sleep 0.2; echo bar")
        .args(&["--prefix-colors", "auto"])
        .stdout();

    let expected_prefix_0 = "\u{1b}[31m[0]\u{1b}[0m";
    let expected_prefix_1 = "\u{1b}[32m[1]\u{1b}[0m";
    let expected_prefix_2 = "\u{1b}[33m[2]\u{1b}[0m";
    let expected = format!(
        "{0} some-file-contents
{0} cat some-file exited with exit status: 0
{1} foo
{1} sleep 0.1; echo foo exited with exit status: 0
{2} bar
{2} sleep 0.2; echo bar exited with exit status: 0
",
        expected_prefix_0, expected_prefix_1, expected_prefix_2
    );

    assert_eq_lines_unordered(escape_debug_by_line(expected), escape_debug_by_line(out));
}

#[test]
fn it_does_not_restart_on_success() {
    let (dir, mut cmd) = setup("it_does_not_restart_on_success");
    dir.create("some-file", "some-file-contents");

    let out = cmd
        .arg("cat some-file")
        .args(&["--restart-tries", "10"])
        .stdout();

    let expected = format!(
        r#"[0] some-file-contents
[0] cat some-file exited with exit status: 0
"#
    );

    assert_eq_lines_unordered(expected, out);
}

#[test]
fn it_supports_restarting() {
    let (_, mut cmd) = setup("it_supports_restarting");
    let out = cmd
        .arg("echo 'hello world'; exit 1")
        .args(&["--restart-tries", "2"])
        .stdout();

    let expected = format!(
        "[0] hello world
[0] echo 'hello world'; exit 1 exited with exit status: 1
[0] echo 'hello world'; exit 1 restarted
[0] hello world
[0] echo 'hello world'; exit 1 exited with exit status: 1
[0] echo 'hello world'; exit 1 restarted
[0] hello world
[0] echo 'hello world'; exit 1 exited with exit status: 1
"
    );

    assert_eq_lines_unordered(expected, out);
}

#[test]
fn it_kills_others_on_exit_0() {
    let (dir, mut cmd) = setup("kill_others_triggers_on_exit_0");
    dir.create("some-file", "some-file-contents");

    let out = cmd
        .arg("exit 0")
        .arg("sleep 0.2; echo 'should not be printed'")
        .arg("--kill-others")
        .stdout();

    let expected = format!(
        r#"[0] exit 0 exited with exit status: 0
--> Sending SIGTERM to other processes..
[1] sleep 0.2; echo 'should not be printed' exited with signal: 15 (SIGTERM)
"#
    );

    assert_eq_lines_unordered(expected, out);
}

#[test]
fn it_doesnt_kill_others_on_fail_on_exit_code_0() {
    let (dir, mut cmd) = setup("kill_others_on_fail_does_not_trigger_on_exit_code_0");
    dir.create("some-file", "some-file-contents");

    let out = cmd
        .arg("exit 0")
        .arg("sleep 0.2; echo 'should be printed'")
        .arg("--kill-others-on-fail")
        .stdout();

    let expected = format!(
        r#"[0] exit 0 exited with exit status: 0
[1] should be printed
[1] sleep 0.2; echo 'should be printed' exited with exit status: 0
"#
    );

    assert_eq_lines_unordered(expected, out);
}

#[test]
fn it_does_kill_others_on_fail_on_exit_code_1() {
    let (dir, mut cmd) = setup("kill_others_on_fail_does_not_trigger_on_exit_code_0");
    dir.create("some-file", "some-file-contents");

    let out = cmd
        .arg("exit 1")
        .arg("sleep 0.2; echo 'should not be printed'")
        .arg("--kill-others-on-fail")
        .stdout();

    let expected = format!(
        r#"[0] exit 1 exited with exit status: 1
--> Sending SIGTERM to other processes..
[1] sleep 0.2; echo 'should not be printed' exited with signal: 15 (SIGTERM)
"#
    );

    assert_eq_lines_unordered(expected, out);
}

#[test]
fn it_supports_hiding() {
    let (dir, mut cmd) = setup("it_supports_hiding");
    dir.create("some-file", "some-file-contents");

    let out = cmd.arg("cat some-file").args(&["--hide", "0"]).stdout();

    // No output if all commands are hidden
    assert_eq_lines_unordered("", out);
}

#[test]
fn it_supports_hiding_by_name() {
    let (dir, mut cmd) = setup("it_supports_hiding_by_name");
    dir.create("some-file", "some-file-contents");

    let out = cmd
        .arg("cat some-file")
        .arg("ls")
        .args(&["--hide", "cat"])
        .args(&["--names", "cat,ls"])
        .stdout();

    let expected = format!(
        r#"[ls] some-file
[ls] ls exited with exit status: 0
"#
    );

    assert_eq_lines_unordered(expected, out);
}

#[cfg(not(windows))]
#[test]
fn it_detects_ctrl_c() {
    let (_, mut cmd) = setup("it_detects_ctrl_c");

    let out = cmd.arg("sleep 5").arg("sleep 10").kill();

    let expected = r#"Ctrl-C issued
Terminating all processes..
[0] sleep 5 exited with signal: 15 (SIGTERM)
[1] sleep 10 exited with signal: 15 (SIGTERM)
"#;

    assert_eq_lines_unordered(expected, out);
}

fn escape_debug_by_line(s: impl AsRef<str>) -> String {
    s.as_ref()
        .escape_debug()
        .to_string()
        .split("\\n")
        .collect::<Vec<_>>()
        .join("\n")
}
