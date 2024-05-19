use crate::utils::*;

#[test]
fn it_starts_with_default_env() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let output = quartz.cmd(&["show", "env"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(output.stdout.trim(), "default");

    Ok(())
}

#[test]
fn it_can_create_env() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let output = quartz.cmd(&["env", "create", "example"])?;
    let list = quartz.cmd(&["env", "list"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert!(
        list.stdout.contains("example"),
        "new env did not show on list"
    );

    Ok(())
}

#[test]
fn it_can_switch_between_envs() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    quartz.cmd(&["env", "create", "example"])?;
    quartz.cmd(&["env", "create", "lorem"])?;

    let assert_use = |name: &str| -> TestResult {
        let output = quartz.cmd(&["env", "use", name])?;
        let status = quartz.cmd(&["show", "env"])?;
        assert!(output.status.success(), "{}", output.stderr);
        assert_eq!(status.stdout.trim(), name, "did not switch to {}", name);

        Ok(())
    };

    assert_use("lorem")?; // Using the newly created env
    assert_use("example")?; // Switch to another one
    assert_use("lorem")?; // Go back to previously selected
    assert_use("default")?; // Going to default

    Ok(())
}

#[test]
fn it_can_not_use_unexistent_env() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let output = quartz.cmd(&["env", "use", "idontexist"])?;
    assert!(
        !output.status.success(),
        "{}",
        "it allowed using unexistent env"
    );

    let status = quartz.cmd(&["show", "env"])?;
    assert_ne!(
        status.stdout.trim(),
        "idontexist",
        "it is using unexistent env"
    );

    Ok(())
}

#[test]
fn it_can_remove_env() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    quartz.cmd(&["env", "create", "example"])?;

    let output = quartz.cmd(&["env", "remove", "example"])?;
    assert!(output.status.success(), "{}", output.stderr);
    let list = quartz.cmd(&["env", "list"])?;
    assert!(
        !list.stdout.contains("example"),
        "example env is still listed"
    );

    Ok(())
}

#[test]
fn it_cannot_remove_unexistent_env() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let output = quartz.cmd(&["env", "remove", "example"])?;
    assert!(!output.status.success(), "did not exit with error");

    Ok(())
}

#[test]
fn it_cannot_create_duplicate() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    quartz.cmd(&["env", "create", "example"])?;
    quartz.cmd(&["env", "create", "lorem"])?;
    let output = quartz.cmd(&["env", "create", "example"])?;

    assert!(
        !output.status.success(),
        "{}",
        "it allowed to create duplicate"
    );

    Ok(())
}

#[test]
fn it_create_headers() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;
    quartz.cmd(&["env", "header", "set", "Header1: Value1"])?;
    quartz.cmd(&["env", "header", "set", "Header2: Value2"])?;

    let output = quartz.cmd(&["env", "header", "get", "Header1"])?;
    assert_eq!(output.stdout.trim(), "Value1");

    let output = quartz.cmd(&["env", "header", "get", "Header2"])?;
    assert_eq!(output.stdout.trim(), "Value2");
    Ok(())
}
