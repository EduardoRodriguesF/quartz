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
