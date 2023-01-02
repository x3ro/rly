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
    let (dir, mut cmd) = setup("it_runs_a_basic_command");
    dir.create("some-file", "some-file-contents");

    let out = cmd
        .arg("cat some-file")
        .args(&["--prefix", "{index}-{command}"])
        .arg("--prefix-length=12")
        .stdout();

    let expected = r#"[0-cat..-file] some-file-contents
[0-cat..-file] cat some-file exited with code 0
"#;

    assert_str_eq!(expected, out);
}
