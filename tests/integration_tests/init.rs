use crate::utils::*;

#[test]
fn it_initializes_quartz() -> TestResult {
    let quartz = Quartz::default();

    let status = quartz.cmd(&["init"])?;

    assert!(status.success(), "init command failed");
    assert!(quartz.dir().exists(), ".quartz was not created");

    Ok(())
}
