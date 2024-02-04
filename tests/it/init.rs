use crate::utils::*;

#[test]
fn it_initializes_quartz() -> TestResult {
    let quartz = Quartz::default();

    let output = quartz.cmd(&["init"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert!(quartz.dir().exists(), ".quartz was not created");

    Ok(())
}

#[test]
fn it_cant_init_over_other() -> TestResult {
    let quartz = Quartz::preset_empty_project()?;

    let output = quartz.cmd(&["init"])?;

    assert!(!output.status.success(), "{}", output.stdout);

    Ok(())
}
