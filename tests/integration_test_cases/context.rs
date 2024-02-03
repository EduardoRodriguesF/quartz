use crate::utils::*;

#[test]
fn it_starts_with_default_context() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let output = quartz.cmd(&["show", "ctx"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(output.stdout.trim(), "default");

    Ok(())
}

#[test]
fn it_can_create_context() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let output = quartz.cmd(&["ctx", "create", "example"])?;
    let list = quartz.cmd(&["ctx", "list"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert!(
        list.stdout.contains("example"),
        "new context did not show on list"
    );

    Ok(())
}

#[test]
fn it_can_switch_between_contexts() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    quartz.cmd(&["ctx", "create", "example"])?;
    quartz.cmd(&["ctx", "create", "lorem"])?;

    let assert_use = |name: &str| -> TestResult {
        let output = quartz.cmd(&["ctx", "use", name])?;
        let status = quartz.cmd(&["show", "ctx"])?;
        assert!(output.status.success(), "{}", output.stderr);
        assert_eq!(status.stdout.trim(), name, "did not switch to {}", name);

        Ok(())
    };

    assert_use("lorem")?; // Using the newly created context
    assert_use("example")?; // Switch to another one
    assert_use("lorem")?; // Go back to previously selected
    assert_use("default")?; // Going to default

    Ok(())
}

#[test]
fn it_can_not_use_unexistent_context() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let output = quartz.cmd(&["ctx", "use", "idontexist"])?;
    assert!(
        !output.status.success(),
        "{}",
        "it allowed using unexistent context"
    );

    let status = quartz.cmd(&["show", "ctx"])?;
    assert_ne!(
        status.stdout.trim(),
        "idontexist",
        "it is using unexistent context"
    );

    Ok(())
}

#[test]
fn it_can_remove_context() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    quartz.cmd(&["context", "create", "example"])?;

    let output = quartz.cmd(&["context", "remove", "example"])?;
    assert!(output.status.success(), "{}", output.stderr);
    let list = quartz.cmd(&["context", "list"])?;
    assert!(
        !list.stdout.contains("example"),
        "example context is still listed"
    );

    Ok(())
}

#[test]
fn it_cannot_remove_unexistent_context() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let output = quartz.cmd(&["context", "remove", "example"])?;
    assert!(!output.status.success(), "did not exit with error");

    Ok(())
}

#[test]
fn it_cannot_create_duplicate() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    quartz.cmd(&["context", "create", "example"])?;
    quartz.cmd(&["context", "create", "lorem"])?;
    let output = quartz.cmd(&["context", "create", "example"])?;

    assert!(
        !output.status.success(),
        "{}",
        "it allowed to create duplicate"
    );

    Ok(())
}
