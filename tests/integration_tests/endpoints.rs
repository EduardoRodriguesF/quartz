use crate::utils::*;

#[test]
fn it_creates_empty_endpoint() -> TestResult {
    let quartz = Quartz::default();
    let sample_endpoint = "myendpoint";

    quartz.cmd(&["init"])?;
    let status = quartz.cmd(&["create", sample_endpoint])?.status;
    let list_bytes = quartz.cmd(&["list"])?.stdout;

    let list = String::from_utf8(list_bytes)?;

    println!("listing: {}", list);

    assert!(status.success(), "create command failed");
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

    let create_result = quartz.cmd(&[
        "create",
        sample_endpoint,
        "--url",
        sample_url,
        "--method",
        "GET",
    ])?;

    quartz.cmd(&["use", sample_endpoint])?;
    let output = quartz.cmd(&["url", "--get"])?;
    let url = String::from_utf8_lossy(&output.stdout);

    assert!(
        create_result.status.success(),
        "{}",
        String::from_utf8_lossy(&create_result.stderr)
    );
    assert_eq!(url.trim(), sample_url);

    Ok(())
}
