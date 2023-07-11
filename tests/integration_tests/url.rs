use crate::utils::*;

#[test]
fn it_gets_endpoint_url() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let url_output = quartz.cmd(&["url", "--get"])?;

    assert_eq!(
        url_output.stdout.trim(),
        "https://httpbin.org/get",
        "{}",
        url_output.stderr
    );

    Ok(())
}

#[test]
fn it_sets_new_endpoint_url() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let set_url_output = quartz.cmd(&["url", "--set", "https://www.google.com/"])?;
    let url_output = quartz.cmd(&["url", "--get"])?;

    assert!(set_url_output.status.success(), "{}", url_output.stderr);
    assert_eq!(
        url_output.stdout.trim(),
        "https://www.google.com/",
        "{}",
        url_output.stderr
    );

    Ok(())
}
