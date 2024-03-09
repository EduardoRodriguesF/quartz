use crate::utils::*;

#[test]
fn it_can_set_query_param() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let set_output = quartz.cmd(&["query", "set", "_v=99890"])?;
    let get_output = quartz.cmd(&["query", "get", "_v"])?;

    assert!(set_output.status.success(), "{}", set_output.stderr);
    assert_eq!(
        get_output.stdout.trim(),
        "99890",
        "did not store query param"
    );

    Ok(())
}

#[test]
fn it_can_set_query_param_with_intentional_equals_sign() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let set_output = quartz.cmd(&["query", "set", "where=email=example@email.com"])?;
    let get_output = quartz.cmd(&["query", "get", "where"])?;

    assert!(set_output.status.success(), "{}", set_output.stderr);
    assert_eq!(
        get_output.stdout.trim(),
        "email=example@email.com",
        "did not store query param correctly"
    );

    Ok(())
}

#[test]
fn it_can_rm_query() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&["query", "set", "fields=example"])?;
    let remove_output = quartz.cmd(&["query", "rm", "fields"])?;
    let get_output = quartz.cmd(&["query", "get", "fields"])?;

    assert!(remove_output.status.success(), "{}", remove_output.stderr);
    assert_eq!(get_output.stdout.trim(), "", "did not removed query");

    Ok(())
}

#[test]
fn it_can_rm_multiple_query() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&[
        "query",
        "set",
        "fields=example",
        "where=email=example@email.com",
        "value=false",
    ])?;
    let remove_output = quartz.cmd(&["query", "rm", "fields", "where"])?;

    let get_output = quartz.cmd(&["query", "get", "fields"])?;
    assert!(remove_output.status.success(), "{}", remove_output.stderr);
    assert_eq!(get_output.stdout.trim(), "", "did not removed query");

    let get_output = quartz.cmd(&["query", "get", "where"])?;
    assert!(remove_output.status.success(), "{}", remove_output.stderr);
    assert_eq!(get_output.stdout.trim(), "", "did not removed query");

    let get_output = quartz.cmd(&["query", "get", "value"])?;
    assert!(remove_output.status.success(), "{}", remove_output.stderr);
    assert_eq!(
        get_output.stdout.trim(),
        "false",
        "removed query that shouldn't"
    );

    Ok(())
}

#[test]
fn rm_multiple_continues_on_err() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    quartz.cmd(&[
        "query",
        "set",
        "fields=example",
        "where=email=example@email.com",
        "value=false",
    ])?;
    let output = quartz.cmd(&["query", "rm", "fields", "idontexist", "where"])?;
    assert!(!output.status.success(), "{}", output.stdout);

    let output = quartz.cmd(&["query", "ls"])?;
    assert!(!output.stdout.contains("fields"), "{}", output.stdout);
    assert!(!output.stdout.contains("where"), "{}", output.stdout);
    assert!(output.stdout.contains("value"), "Unproperly removed fields");

    Ok(())
}

#[test]
fn it_outputs_resolved_string() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&[
        "query",
        "set",
        "_v=99890",
        "fields=lorem,ipsum",
        "helloString=true",
    ])?;

    let output = quartz.cmd(&["show", "query"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(
        output.stdout.trim(),
        "_v=99890&fields=lorem,ipsum&helloString=true",
        "did not match queries"
    );

    Ok(())
}

#[test]
fn compatible_with_apply_env_option() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&["var", "set", "someQuery=yay"])?;
    quartz.cmd(&["query", "set", "version={{someQuery}}"])?;

    let output = quartz.cmd(&["--apply-environment", "query", "get", "version"])?;
    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(output.stdout.trim(), "yay");

    let output = quartz.cmd(&["--apply-environment", "query", "ls"])?;
    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(output.stdout.trim(), "version=yay");

    Ok(())
}
