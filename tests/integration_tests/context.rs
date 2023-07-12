use crate::utils::*;

#[test]
fn it_starts_with_default_context() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let output = quartz.cmd(&["status", "--context"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(output.stdout.trim(), "default");

    Ok(())
}

#[test]
fn it_can_create_context() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let output = quartz.cmd(&["context", "create", "example"])?;
    let list = quartz.cmd(&["context", "list"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert!(
        list.stdout.contains("example"),
        "new context did not show on list"
    );

    Ok(())
}

#[test]
fn it_can_switch_between_contexts() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&["context", "create", "example"])?;
    quartz.cmd(&["context", "create", "lorem"])?;

    let assert_use = |name: &str| -> TestResult {
        let output = quartz.cmd(&["context", "use", name])?;
        let status = quartz.cmd(&["status", "--context"])?;
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
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let output = quartz.cmd(&["context", "use", "idontexist"])?;
    assert!(
        !output.status.success(),
        "{}",
        "it allowed using unexistent context"
    );

    let status = quartz.cmd(&["status", "--context"])?;
    assert_ne!(
        status.stdout.trim(),
        "idontexist",
        "it is using unexistent context"
    );

    Ok(())
}
