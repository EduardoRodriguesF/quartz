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

#[test]
fn init_detect_git() -> TestResult {
    let quartz = Quartz::default();
    let binding = quartz.dir();
    let parent = binding.parent().unwrap();

    // Fake .git
    std::fs::create_dir(parent.join(".git"))?;

    let output = quartz.cmd(&["init"])?;

    assert!(output.status.success(), "{}", output.stderr);
    assert!(quartz.dir().exists(), ".quartz was not created");
    assert!(parent.join(".gitignore").exists(), ".git was not created");

    let gitignore = std::fs::read_to_string(parent.join(".gitignore"))?;
    assert!(
        gitignore.contains(".quartz/user"),
        ".gitignore does not contain .quartz/user"
    );

    Ok(())
}
