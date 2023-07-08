mod utils;

use utils::*;

#[test]
fn init_quartz() -> TestResult {
    let quartz = Quartz::default();

    let status = quartz.cmd(&["init"])?;

    assert!(status.success(), "init command failed");
    assert!(quartz.dir().exists(), ".quartz was not created");

    Ok(())
}
