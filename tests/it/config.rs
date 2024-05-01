use crate::utils::*;

#[test]
fn it_can_get_values() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let output = quartz.cmd(&["config", "get", "ui.colors"])?;
    assert!(output.status.success(), "command exit with error");

    Ok(())
}

#[test]
fn get_fails_on_invalid_key() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let output = quartz.cmd(&["config", "get", "no.colors"])?;
    assert!(!output.status.success(), "command exit without error");

    Ok(())
}

#[test]
fn it_can_set_value() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let set_output = quartz.cmd(&["config", "set", "preferences.editor", "emacs"])?;
    let get_output = quartz.cmd(&["config", "get", "preferences.editor"])?;

    assert!(set_output.status.success(), "{}", set_output.stderr);
    assert_eq!(get_output.stdout.trim(), "emacs", "did not save new value");

    Ok(())
}

#[test]
fn it_can_not_set_invalid_key() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let set_output = quartz.cmd(&["config", "set", "example.editor", "emacs"])?;
    assert!(!set_output.status.success(), "command exit without error");

    Ok(())
}
