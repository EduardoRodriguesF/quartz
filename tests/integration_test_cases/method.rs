use crate::utils::*;

#[test]
fn it_gets_endpoint_method() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let method_output = quartz.cmd(&["method", "--get"])?;

    assert_eq!(
        method_output.stdout.trim(),
        "GET",
        "{}",
        method_output.stderr
    );

    Ok(())
}

#[test]
fn it_sets_new_endpoint_method() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let set_method_output = quartz.cmd(&["method", "--set", "POST"])?;
    let method_output = quartz.cmd(&["method", "--get"])?;

    assert!(
        set_method_output.status.success(),
        "{}",
        method_output.stderr
    );
    assert_eq!(
        method_output.stdout.trim(),
        "POST",
        "{}",
        method_output.stderr
    );

    Ok(())
}
