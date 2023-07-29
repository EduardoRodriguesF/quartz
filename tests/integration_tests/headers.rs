use crate::utils::*;

#[test]
fn it_adds_new_header() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let set_output = quartz.cmd(&["header", "--set", "Content-type: application/json"])?;
    let output = quartz.cmd(&["header", "--get", "Content-type"])?;

    assert!(set_output.status.success(), "{}", set_output.stderr);
    assert_eq!(output.stdout.trim(), "application/json");

    Ok(())
}

#[test]
fn it_adds_multiple_new_header() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let sample_headers = vec![
        "Content-type: application/json",
        "Accept: application/json",
        "X-API-key: myexample",
    ];

    let headers_add_output = quartz.cmd(&[
        "header",
        "--set",
        sample_headers[0],
        "--set",
        sample_headers[1],
        "--set",
        sample_headers[2],
    ])?;
    let output = quartz.cmd(&["header", "--list"])?;

    assert!(
        headers_add_output.status.success(),
        "{}",
        headers_add_output.stdout
    );

    for header in sample_headers {
        assert!(output.stdout.contains(header));
    }

    Ok(())
}

#[test]
fn it_overwrites_existing_headers() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&[
        "header",
        "--set",
        "Content-type: application/json",
        "--set",
        "Accept: application/json",
    ])?;

    let edit_output = quartz.cmd(&["header", "--set", "Content-type: plain/text"])?;
    let output = quartz.cmd(&["header", "--list"])?;

    assert!(edit_output.status.success(), "{}", edit_output.stdout);

    assert!(
        !output.stdout.contains("Content-type: application/json"),
        "old value found"
    );
    assert!(
        output.stdout.contains("Content-type: plain/text"),
        "new value was not saved"
    );

    Ok(())
}

#[test]
fn it_removes_header_by_key() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&[
        "header",
        "--set",
        "Content-type: application/json",
        "--set",
        "Accept: form",
    ])?;

    let remove_output = quartz.cmd(&["header", "--remove", "Content-type", "--list"])?;
    assert!(remove_output.status.success(), "{}", remove_output.stderr);
    assert!(
        !remove_output.stdout.contains("Content-type"),
        "did not remove specified header"
    );
    assert!(
        remove_output.stdout.contains("Accept"),
        "removed specified header, but unrelated header is missing"
    );

    Ok(())
}

#[test]
fn it_does_not_allow_invalid_header_format() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let output = quartz.cmd(&["header", "--set", "Content-type"])?;
    assert!(
        !output.status.success(),
        "allowed header without value separation"
    );

    let output = quartz.cmd(&["header", "--set", "Content-type = application/json"])?;
    assert!(
        !output.status.success(),
        "allowed header with incorrect key-value separation"
    );

    let output = quartz.cmd(&["headers", "--set", "Content-type:application/json"])?;
    assert!(
        !output.status.success(),
        "allowed header without proper spacing between key and value"
    );

    Ok(())
}

#[test]
fn compatible_with_apply_context_option() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&["var", "--set", "contentType=application/json"])?;
    quartz.cmd(&["header", "--set", "Content-type: {{contentType}}"])?;

    let output = quartz.cmd(&["--apply-context", "header", "--get", "Content-type"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(output.stdout.trim(), "application/json");

    Ok(())
}
