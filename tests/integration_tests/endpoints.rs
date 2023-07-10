use crate::utils::*;

#[test]
fn it_creates_empty_endpoint() -> TestResult {
    let quartz = Quartz::default();
    let sample_endpoint = "myendpoint";

    quartz.cmd(&["init"])?;
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
    let quartz = Quartz::default();
    let sample_endpoint = "myendpoint";
    let sample_url = "https://httpbin.org/get";

    quartz.cmd(&["init"])?;

    let create_output = quartz.cmd(&["create", sample_endpoint, "--url", sample_url])?;

    assert!(create_output.status.success(), "{}", create_output.stderr);

    quartz.cmd(&["use", sample_endpoint])?;
    let url_output = quartz.cmd(&["url", "--get"])?;

    assert_eq!(url_output.stdout.trim(), sample_url.trim());

    Ok(())
}

#[test]
fn it_creates_endpoint_with_method() -> TestResult {
    let quartz = Quartz::default();
    let sample_endpoint = "myendpoint";
    let sample_url = "https://httpbin.org/get";
    let method = "POST";

    quartz.cmd(&["init"])?;

    let create_output = quartz.cmd(&[
        "create",
        sample_endpoint,
        "--url",
        sample_url,
        "--method",
        method,
    ])?;

    assert!(create_output.status.success(), "{}", create_output.stderr);

    quartz.cmd(&["use", sample_endpoint])?;
    let method_output = quartz.cmd(&["method", "--get"])?;

    assert_eq!(method_output.stdout.trim(), method);

    Ok(())
}

#[test]
fn it_creates_endpoint_with_header() -> TestResult {
    let quartz = Quartz::default();
    let sample_endpoint = "myendpoint";

    quartz.cmd(&["init"])?;

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
    let method_output = quartz.cmd(&["headers", "--list"])?;

    assert!(method_output
        .stdout
        .contains("Content-type: application/json"));

    Ok(())
}

#[test]
fn it_creates_endpoint_with_multiple_headers() -> TestResult {
    let quartz = Quartz::default();
    let sample_endpoint = "myendpoint";

    quartz.cmd(&["init"])?;

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
    let method_output = quartz.cmd(&["headers", "--list"])?;

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
fn it_does_not_allow_create_without_reference() -> TestResult {
    let quartz = Quartz::default();

    quartz.cmd(&["init"])?;

    let create_output = quartz.cmd(&["create"])?;

    assert!(
        !create_output.status.success(),
        "created endpoint without reference"
    );

    Ok(())
}

#[test]
fn it_does_not_allow_create_duplicate() -> TestResult {
    let quartz = Quartz::default();
    quartz.cmd(&["init"])?;

    quartz.cmd(&["create", "myendpoint", "--url", "https://original/"])?;
    let duplicate_create_output =
        quartz.cmd(&["create", "myendpoint", "--url", "https://overwritten/"])?;

    quartz.cmd(&["use", "myendpoint"])?;
    let url_output = quartz.cmd(&["url", "--get"])?;

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
