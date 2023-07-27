use crate::utils::*;

#[test]
fn it_can_set_query_param() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let set_output = quartz.cmd(&["query", "--set", "_v=99890"])?;
    let get_output = quartz.cmd(&["query", "--get", "_v"])?;

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

    let set_output = quartz.cmd(&["query", "--set", "where=email=example@email.com"])?;
    let get_output = quartz.cmd(&["query", "--get", "where"])?;

    assert!(set_output.status.success(), "{}", set_output.stderr);
    assert_eq!(
        get_output.stdout.trim(),
        "email=example@email.com",
        "did not store query param correctly"
    );

    Ok(())
}

#[test]
fn it_can_remove_query() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&["query", "--set", "fields=example"])?;
    let remove_output = quartz.cmd(&["query", "--remove", "fields"])?;
    let get_output = quartz.cmd(&["query", "--get", "fields"])?;

    assert!(remove_output.status.success(), "{}", remove_output.stderr);
    assert_eq!(get_output.stdout.trim(), "", "did not removed query");

    Ok(())
}

#[test]
fn it_outputs_resolved_string() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&["query", "--set", "_v=99890"])?;
    quartz.cmd(&["query", "--set", "fields=lorem,ipsum"])?;
    quartz.cmd(&["query", "--set", "helloString=true"])?;

    let output = quartz.cmd(&["query", "--get"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(
        output.stdout.trim(),
        "_v=99890&fields=lorem,ipsum&helloString=true",
        "did not match queries"
    );

    Ok(())
}

#[test]
fn it_outputs_full_url() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&["query", "--set", "_v=99890"])?;
    quartz.cmd(&["query", "--set", "fields=lorem,ipsum"])?;

    let output = quartz.cmd(&["url", "--get", "--full"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(
        output.stdout.trim(),
        "https://httpbin.org/get?_v=99890&fields=lorem,ipsum"
    );

    Ok(())
}

#[test]
fn it_outputs_full_url_with_messy_queries() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&["url", "--set", "https://httpbin.org/get?value=true"])?;
    quartz.cmd(&["query", "--set", "_v=99890"])?;
    quartz.cmd(&["query", "--set", "fields=lorem,ipsum"])?;

    let output = quartz.cmd(&["url", "--get", "--full"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(
        output.stdout.trim(),
        "https://httpbin.org/get?value=true&_v=99890&fields=lorem,ipsum"
    );

    Ok(())
}

#[test]
fn compatible_with_apply_context_option() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&["var", "--set", "someQuery=yay"])?;

    quartz.cmd(&["query", "--set", "version={{someQuery}}"])?;

    let output = quartz.cmd(&["query", "--get", "version"])?;
    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(output.stdout.trim(), "yay");

    let output = quartz.cmd(&["query", "--get"])?;
    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(output.stdout.trim(), "version=yay");

    Ok(())
}
