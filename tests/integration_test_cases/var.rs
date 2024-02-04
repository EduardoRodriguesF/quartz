use crate::utils::*;

#[test]
fn it_can_create_variables() -> TestResult {
    let quartz = Quartz::preset_using_default_env()?;

    let output = quartz.cmd(&["var", "set", "baseUrl=localhost"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let output = quartz.cmd(&["var", "get", "baseUrl"])?;
    assert_eq!(
        output.stdout.trim(),
        "localhost",
        "did not save variable correctly"
    );

    Ok(())
}

#[test]
fn it_can_set_multiple_variables() -> TestResult {
    let quartz = Quartz::preset_using_default_env()?;

    let output = quartz.cmd(&["var", "set", "baseUrl=localhost", "scheme=https"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let output = quartz.cmd(&["variable", "get", "baseUrl"])?;
    assert_eq!(
        output.stdout.trim(),
        "localhost",
        "did not save first variable correctly"
    );

    let output = quartz.cmd(&["variable", "get", "scheme"])?;
    assert_eq!(
        output.stdout.trim(),
        "https",
        "did not save second variable correctly"
    );

    Ok(())
}
#[test]
fn it_ignores_outer_single_quotes() -> TestResult {
    let quartz = Quartz::preset_using_default_env()?;

    let set_output = quartz.cmd(&["variable", "set", "baseUrl='localhost'"])?;
    let get_output = quartz.cmd(&["variable", "get", "baseUrl"])?;

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
    let quartz = Quartz::preset_using_default_env()?;

    let set_output = quartz.cmd(&["variable", "set", "baseUrl=\"localhost\""])?;
    let get_output = quartz.cmd(&["variable", "get", "baseUrl"])?;

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
    let quartz = Quartz::preset_using_default_env()?;

    quartz.cmd(&["variable", "set", "baseUrl=localhost"])?;

    let set_output = quartz.cmd(&["variable", "set", "baseUrl=128.0.0.1"])?;
    let get_output = quartz.cmd(&["variable", "get", "baseUrl"])?;

    assert!(set_output.status.success(), "{}", set_output.stderr);
    assert_eq!(
        get_output.stdout.trim(),
        "128.0.0.1",
        "did not overwrote variable value"
    );

    Ok(())
}

#[test]
fn each_env_has_its_own_variables() -> TestResult {
    let quartz = Quartz::preset_using_default_env()?;

    quartz.cmd(&["variable", "set", "baseUrl=localhost"])?;
    quartz.cmd(&["env", "create", "example"])?;
    quartz.cmd(&["env", "use", "example"])?;

    let output = quartz.cmd(&["variable", "get", "baseUrl"])?;
    assert_ne!(output.stdout.trim(), "localhost", "");

    quartz.cmd(&["env", "use", "default"])?;

    let output = quartz.cmd(&["variable", "get", "baseUrl"])?;
    assert_eq!(output.stdout.trim(), "localhost");

    Ok(())
}

#[test]
fn it_can_remove_variable() -> TestResult {
    let quartz = Quartz::preset_using_default_env()?;

    quartz.cmd(&["var", "set", "baseUrl=localhost"])?;
    let output = quartz.cmd(&["var", "rm", "baseUrl"])?;
    let get_output = quartz.cmd(&["var", "get", "baseUrl"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert!(get_output.stdout.is_empty(), "{:?}", output.stdout);

    Ok(())
}

#[test]
fn it_can_remove_multiple_variable() -> TestResult {
    let quartz = Quartz::preset_using_default_env()?;

    quartz.cmd(&["var", "set", "baseUrl=localhost", "other=true", "flag=on"])?;
    let output = quartz.cmd(&["var", "rm", "baseUrl", "flag"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let output = quartz.cmd(&["var", "ls"])?;
    assert!(output.stdout.contains("other=true"), "{}", output.stdout);
    assert!(!output.stdout.contains("baseUrl"), "{}", output.stdout);
    assert!(!output.stdout.contains("flag"), "{}", output.stdout);

    Ok(())
}
