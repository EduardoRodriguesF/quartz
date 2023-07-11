use crate::utils::*;

#[test]
fn it_adds_new_header() -> TestResult {
    let quartz = Quartz::default();
    let sample_endpoint = "myendpoint";

    quartz.cmd(&["init"])?;

    quartz.cmd(&[
        "create",
        sample_endpoint,
        "--url",
        "https://httpbin.org/get",
    ])?;

    quartz.cmd(&["use", sample_endpoint])?;
    let headers_add_output = quartz.cmd(&["headers", "--add", "Content-type: application/json"])?;
    let headers_output = quartz.cmd(&["headers", "--list"])?;

    assert!(
        headers_add_output.status.success(),
        "{}",
        headers_add_output.stdout
    );
    assert!(headers_output
        .stdout
        .contains("Content-type: application/json"));

    Ok(())
}

#[test]
fn it_adds_multiple_new_header() -> TestResult {
    let quartz = Quartz::default();
    let sample_endpoint = "myendpoint";

    quartz.cmd(&["init"])?;

    quartz.cmd(&[
        "create",
        sample_endpoint,
        "--url",
        "https://httpbin.org/get",
    ])?;

    quartz.cmd(&["use", sample_endpoint])?;

    let sample_headers = vec![
        "Content-type: application/json",
        "Accept: application/json",
        "X-API-key: myexample",
    ];

    let headers_add_output = quartz.cmd(&[
        "headers",
        "--add",
        sample_headers[0],
        "--add",
        sample_headers[1],
        "--add",
        sample_headers[2],
    ])?;
    let headers_output = quartz.cmd(&["headers", "--list"])?;

    assert!(
        headers_add_output.status.success(),
        "{}",
        headers_add_output.stdout
    );

    for header in sample_headers {
        assert!(headers_output.stdout.contains(header));
    }

    Ok(())
}

#[test]
fn it_overwrites_existing_headers() -> TestResult {
    let quartz = Quartz::default();
    let sample_endpoint = "myendpoint";

    quartz.cmd(&["init"])?;

    quartz.cmd(&[
        "create",
        sample_endpoint,
        "--url",
        "https://httpbin.org/get",
    ])?;

    quartz.cmd(&["use", sample_endpoint])?;

    quartz.cmd(&[
        "headers",
        "--add",
        "Content-type: application/json",
        "--add",
        "Accept: application/json",
    ])?;

    let headers_edit_output = quartz.cmd(&["headers", "--add", "Content-type: plain/text"])?;
    let headers_output = quartz.cmd(&["headers", "--list"])?;

    assert!(
        headers_edit_output.status.success(),
        "{}",
        headers_edit_output.stdout
    );

    assert!(
        !headers_output
            .stdout
            .contains("Content-type: application/json"),
        "old value found"
    );
    assert!(
        headers_output.stdout.contains("Content-type: plain/text"),
        "new value was not saved"
    );

    Ok(())
}
