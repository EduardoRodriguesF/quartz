use crate::utils::*;

#[test]
fn it_initializes_quartz() -> TestResult {
    let quartz = Quartz::default();

    let output = quartz.cmd(&["init"])?;

    assert!(output.status.success(), "{}", output.stdout);
    assert!(quartz.dir().exists(), ".quartz was not created");

    Ok(())
}
