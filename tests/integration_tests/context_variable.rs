use crate::utils::*;

#[test]
fn it_can_create_variables() -> TestResult {
    let quartz = Quartz::preset_using_default_context()?;

    let output = quartz.cmd(&["variable", "--set", "baseUrl=localhost"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let output = quartz.cmd(&["variable", "--get", "baseUrl"])?;
    assert_eq!(
        output.stdout.trim(),
        "localhost",
        "did not save variable correctly"
    );

    Ok(())
}

#[test]
fn it_ignores_outer_single_quotes() -> TestResult {
    let quartz = Quartz::preset_using_default_context()?;

    let set_output = quartz.cmd(&["variable", "--set", "baseUrl='localhost'"])?;
    let get_output = quartz.cmd(&["variable", "--get", "baseUrl"])?;

    assert!(set_output.status.success(), "{}", set_output.stderr);
    assert_eq!(
        get_output.stdout.trim(),
        "localhost",
        "did not save variable correctly"
    );

    Ok(())
}

#[test]
fn it_ignores_outer_double_quotes() -> TestResult {
    let quartz = Quartz::preset_using_default_context()?;

    let set_output = quartz.cmd(&["variable", "--set", "baseUrl=\"localhost\""])?;
    let get_output = quartz.cmd(&["variable", "--get", "baseUrl"])?;

    assert!(set_output.status.success(), "{}", set_output.stderr);
    assert_eq!(
        get_output.stdout.trim(),
        "localhost",
        "did not save variable correctly"
    );

    Ok(())
}

#[test]
fn it_can_overwrite_existing_variables() -> TestResult {
    let quartz = Quartz::preset_using_default_context()?;

    quartz.cmd(&["variable", "--set", "baseUrl=localhost"])?;

    let set_output = quartz.cmd(&["variable", "--set", "baseUrl=128.0.0.1"])?;
    let get_output = quartz.cmd(&["variable", "--get", "baseUrl"])?;

    assert!(set_output.status.success(), "{}", set_output.stderr);
    assert_eq!(
        get_output.stdout.trim(),
        "128.0.0.1",
        "did not overwrote variable value"
    );

    Ok(())
}

#[test]
fn each_context_has_its_own_variables() -> TestResult {
    let quartz = Quartz::preset_using_default_context()?;

    quartz.cmd(&["variable", "--set", "baseUrl=localhost"])?;
    quartz.cmd(&["context", "create", "example"])?;
    quartz.cmd(&["context", "use", "example"])?;

    let output = quartz.cmd(&["variable", "--get", "baseUrl"])?;
    assert_ne!(output.stdout.trim(), "localhost", "");

    quartz.cmd(&["context", "use", "default"])?;

    let output = quartz.cmd(&["variable", "--get", "baseUrl"])?;
    assert_eq!(output.stdout.trim(), "localhost");

    Ok(())
}
