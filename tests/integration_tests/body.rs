use crate::utils::*;

const SAMPLE_BODY_1: &'static str = r#"
{
    "example": 123
}
"#;

const SAMPLE_BODY_2: &'static str = r#"
{
    "prop": {
        "hello": 123
    },
    "sample": "lorem ipsum"
}
"#;

#[test]
fn it_accepts_json_from_stdin() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let output = quartz.cmd_stdin(&["body", "--stdin"], SAMPLE_BODY_1)?;
    assert!(output.status.success(), "{}", output.stderr);

    let output = quartz.cmd(&["body", "--print"])?;
    assert_eq!(output.stdout.trim(), SAMPLE_BODY_1.trim());

    Ok(())
}

#[test]
fn it_chains_stdin_and_print() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let output = quartz.cmd_stdin(&["body", "--stdin", "--print"], SAMPLE_BODY_1)?;
    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(output.stdout.trim(), SAMPLE_BODY_1.trim());

    Ok(())
}

#[test]
fn it_changes_existent_body() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd_stdin(&["body", "--stdin"], SAMPLE_BODY_1)?;
    let output = quartz.cmd_stdin(&["body", "--stdin", "--print"], SAMPLE_BODY_2)?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_ne!(
        output.stdout.trim(),
        SAMPLE_BODY_1.trim(),
        "it preserved the first body"
    );
    assert_eq!(
        output.stdout.trim(),
        SAMPLE_BODY_2.trim(),
        "did not update body with correct value"
    );

    Ok(())
}
