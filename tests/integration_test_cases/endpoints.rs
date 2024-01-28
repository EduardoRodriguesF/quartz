use crate::utils::*;

#[test]
fn it_creates_empty_endpoint() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;
    let sample_endpoint = "myendpoint";

    let output = quartz.cmd(&["create", sample_endpoint])?;
    let list = quartz.cmd(&["list"])?.stdout;

    assert!(output.status.success(), "{}", output.stderr);
    assert!(
        list.contains(sample_endpoint),
        "Endpoint was not properly created"
    );

    Ok(())
}

#[test]
fn it_creates_endpoint_with_url() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;
    let sample_endpoint = "myendpoint";
    let sample_url = "https://httpbin.org/get";

    let create_output = quartz.cmd(&["create", sample_endpoint, "--url", sample_url])?;

    assert!(create_output.status.success(), "{}", create_output.stderr);

    quartz.cmd(&["use", sample_endpoint])?;
    let url_output = quartz.cmd(&["show", "url"])?;

    assert_eq!(url_output.stdout.trim(), sample_url.trim());

    Ok(())
}

#[test]
fn it_creates_endpoint_with_method() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;
    let sample_endpoint = "myendpoint";
    let sample_url = "https://httpbin.org/get";
    let method = "POST";

    let create_output =
        quartz.cmd(&["create", sample_endpoint, "--url", sample_url, "-X", method])?;

    assert!(create_output.status.success(), "{}", create_output.stderr);

    quartz.cmd(&["use", sample_endpoint])?;
    let method_output = quartz.cmd(&["show", "method"])?;

    assert_eq!(method_output.stdout.trim(), method);

    Ok(())
}

#[test]
fn it_creates_endpoint_with_header() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;
    let sample_endpoint = "myendpoint";

    let create_output = quartz.cmd(&[
        "create",
        sample_endpoint,
        "--url",
        sample_endpoint,
        "--header",
        "Content-type: application/json",
    ])?;

    assert!(create_output.status.success(), "{}", create_output.stderr);

    quartz.cmd(&["use", sample_endpoint])?;
    let method_output = quartz.cmd(&["header", "ls"])?;

    assert!(method_output
        .stdout
        .contains("Content-type: application/json"));

    Ok(())
}

#[test]
fn it_creates_endpoint_with_query() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;
    let sample_endpoint = "myendpoint";

    let create_output = quartz.cmd(&[
        "create",
        sample_endpoint,
        "--url",
        sample_endpoint,
        "--query",
        "myvariable=true",
    ])?;

    assert!(create_output.status.success(), "{}", create_output.stderr);

    quartz.cmd(&["use", sample_endpoint])?;
    let output = quartz.cmd(&["query", "get", "myvariable"])?;

    assert_eq!(output.stdout.trim(), "true");

    Ok(())
}

#[test]
fn it_creates_endpoint_with_multiple_headers() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;
    let sample_endpoint = "myendpoint";

    let create_output = quartz.cmd(&[
        "create",
        sample_endpoint,
        "--url",
        sample_endpoint,
        "--header",
        "Content-type: application/json",
        "--header",
        "Accept: application/json",
    ])?;

    assert!(create_output.status.success(), "{}", create_output.stderr);

    quartz.cmd(&["use", sample_endpoint])?;
    let method_output = quartz.cmd(&["header", "ls"])?;

    assert!(
        method_output
            .stdout
            .contains("Content-type: application/json"),
        "missing first header"
    );
    assert!(
        method_output.stdout.contains("Accept: application/json"),
        "missing second header"
    );

    Ok(())
}

#[test]
fn it_creates_nested_endpoints() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let output = quartz.cmd(&[
        "create",
        "myendpoint/childendpoint",
        "--url",
        "https://this-is-the-nested-one.com",
    ])?;
    assert!(output.status.success(), "{}", output.stderr);

    let output = quartz.cmd(&["use", "myendpoint/childendpoint"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let output = quartz.cmd(&["show", "url"])?;
    assert_eq!(
        output.stdout.trim(),
        "https://this-is-the-nested-one.com",
        "could not use nested endpoint"
    );

    Ok(())
}

#[test]
fn it_does_not_allow_create_without_handle() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let create_output = quartz.cmd(&["create"])?;

    assert!(
        !create_output.status.success(),
        "created endpoint without handle"
    );

    Ok(())
}

#[test]
fn it_does_not_allow_create_duplicate() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    quartz.cmd(&["create", "myendpoint", "--url", "https://original/"])?;
    let duplicate_create_output =
        quartz.cmd(&["create", "myendpoint", "--url", "https://overwritten/"])?;

    quartz.cmd(&["use", "myendpoint"])?;
    let url_output = quartz.cmd(&["show", "url"])?;

    assert_ne!(
        url_output.stdout.trim(),
        "https://overwritten/",
        "duplicate overwrote original endpoint data"
    );

    assert!(
        !duplicate_create_output.status.success(),
        "created duplicate endpoint"
    );

    Ok(())
}

#[test]
fn it_can_run_from_another_handle() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&["create", "anotherendpoint"])?;

    let output = quartz.cmd(&["-x", "anotherendpoint", "status", "--endpoint"])?;
    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(
        output.stdout.trim(),
        "anotherendpoint",
        "did not use desired handle"
    );

    let output = quartz.cmd(&["status", "--endpoint"])?;
    assert_ne!(
        output.stdout.trim(),
        "anotherendpoint",
        "changed endpoint state instead of only running once"
    );

    Ok(())
}

#[test]
fn use_can_set_properties() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let output = quartz.cmd(&[
        "use",
        "-X",
        "POST",
        "-H",
        "Content-type: application/json",
        "-H",
        "Accept: application/json",
        "-q",
        "value=true",
    ])?;

    assert!(output.status.success(), "{}", output.stderr);

    Ok(())
}
