use crate::utils::{Quartz, TestResult};

#[test]
pub fn cp() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["cp", "httpbin/post", "httpbin/patch"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let output = quartz.cmd(&["-x", "httpbin/patch", "show", "url"])?;
    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(
        output.stdout.trim(),
        "{{BASE_URL}}/post",
        "{}",
        output.stderr
    );
    let output = quartz.cmd(&["-x", "httpbin/patch", "show", "method"])?;
    assert_eq!(output.stdout.trim(), "POST", "{}", output.stdout);

    Ok(())
}

/// Without `--recursive` option, only the specified handle should be copied. Child endpoints should be ignored.
#[test]
pub fn cp_wo_r_only_copies_specified() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["cp", "httpbin/redirect", "httpbin/recurse"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let stdout = quartz.cmd(&["ls"])?.stdout;
    assert!(stdout.contains("httpbin/recurse"), "{}", stdout);
    assert!(!stdout.contains("httpbin/recurse/absolute"), "{}", stdout);
    assert!(!stdout.contains("httpbin/recurse/relative"), "{}", stdout);

    Ok(())
}

#[test]
pub fn cp_r() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["cp", "-r", "httpbin/redirect", "httpbin/recurse"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let stdout = quartz.cmd(&["ls"])?.stdout;
    assert!(stdout.contains("httpbin/recurse"), "{}", stdout);
    assert!(stdout.contains("httpbin/recurse/absolute"), "{}", stdout);
    assert!(stdout.contains("httpbin/recurse/relative"), "{}", stdout);

    Ok(())
}

/// Should be able to copy empty handles. Useful for copying groups that start
/// in such.
#[test]
pub fn cp_empty() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["cp", "-r", "httpbin", "https"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let stdout = quartz.cmd(&["ls"])?.stdout;
    assert!(stdout.contains("https/get"), "{}", stdout);
    assert!(stdout.contains("https/post"), "{}", stdout);
    assert!(stdout.contains("https/redirect"), "{}", stdout);
    assert!(stdout.contains("https/redirect/absolute"), "{}", stdout);
    assert!(stdout.contains("https/redirect/relative"), "{}", stdout);

    Ok(())
}

#[test]
pub fn rm() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["rm", "httpbin/post"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let output = quartz.cmd(&["use", "httpbin/post"])?;
    assert!(!output.status.success(), "{}", output.stdout);

    Ok(())
}

#[test]
pub fn rm_wo_r_err() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["rm", "httpbin/redirect"])?;
    assert!(!output.status.success(), "{}", output.stdout);

    let stdout = quartz.cmd(&["ls"])?.stdout;
    assert!(stdout.contains("httpbin/redirect"), "{}", stdout);
    assert!(stdout.contains("httpbin/redirect/absolute"), "{}", stdout);
    assert!(stdout.contains("httpbin/redirect/relative"), "{}", stdout);

    Ok(())
}

#[test]
pub fn rm_r() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["rm", "-r", "httpbin/redirect"])?;
    assert!(!output.status.success(), "{}", output.stdout);

    let stdout = quartz.cmd(&["ls"])?.stdout;
    assert!(!stdout.contains("httpbin/redirect"), "{}", stdout);

    Ok(())
}

#[test]
pub fn rm_multiple() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&[
        "rm",
        "httpbin/get",
        "httpbin/post",
        "httpbin/redirect/absolute",
    ])?;
    assert!(output.status.success(), "{}", output.stderr);

    let stdout = quartz.cmd(&["ls"])?.stdout;
    assert!(!stdout.contains("httpbin/get"), "{}", stdout);
    assert!(!stdout.contains("httpbin/post"), "{}", stdout);
    assert!(!stdout.contains("httpbin/redirect/absolute"), "{}", stdout);

    Ok(())
}

/// The execution of rm should not stop if one of the handles don't exist.
///
/// ```bash
/// quartz rm idontexist httpbin/post idontexist2 httpbin/get
/// ```
///
/// Should successfully remove httpbin/post and httpbin/get, but return a non-zero exit code.
#[test]
pub fn rm_multiple_continues_on_err() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&[
        "rm",
        "httpbin/get",
        "idontexist",
        "httpbin/post",
        "httpbin/redirect/absolute",
    ])?;
    assert!(!output.status.success(), "{}", output.stdout);

    let output = quartz.cmd(&["ls"])?;
    assert!(!output.stdout.contains("httpbin/get"), "{}", output.stdout);
    assert!(!output.stdout.contains("httpbin/post"), "{}", output.stdout);
    assert!(
        !output.stdout.contains("httpbin/redirect/absolute"),
        "{}",
        output.stdout
    );

    Ok(())
}

#[test]
pub fn mv() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["mv", "httpbin/get", "httpbin/getter"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let output = quartz.cmd(&["use", "httpbin/get"])?;
    assert!(!output.status.success(), "httpbin/get should not exist");

    let output = quartz.cmd(&["-x", "httpbin/getter", "show", "url"])?;
    assert_eq!(output.stdout.trim(), "{{BASE_URL}}/get");

    Ok(())
}

#[test]
pub fn mv_overwrite_empty() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["mv", "httpbin/get", "httpbin"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let output = quartz.cmd(&["use", "httpbin/get"])?;
    assert!(!output.status.success(), "httpbin/get should not exist");

    let output = quartz.cmd(&["-x", "httpbin", "show", "url"])?;
    assert_eq!(output.stdout.trim(), "{{BASE_URL}}/get");

    Ok(())
}

/// When moving a single endpoint to an existent endpoint, overwrite it.
#[test]
pub fn mv_overwrite() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["mv", "httpbin/get", "httpbin"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let output = quartz.cmd(&["use", "httpbin/get"])?;
    assert!(!output.status.success(), "httpbin/get should not exist");

    let output = quartz.cmd(&["-x", "httpbin", "show", "url"])?;
    assert_eq!(output.stdout.trim(), "{{BASE_URL}}/get");

    Ok(())
}

/// When selecting multiple handles to mv at once, it should nest them into the last argument.
///
/// ```bash
/// quartz mv e1 e2 group
/// ```
///
/// No matter if `group` exists or not, it should result in:
/// * group
///     * e1
///     * e2
#[test]
pub fn mv_multiple() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&[
        "mv",
        "httpbin/redirect/absolute",
        "httpbin/redirect/relative",
        "httpbin",
    ])?;
    assert!(output.status.success(), "{}", output.stderr);

    let stdout = quartz.cmd(&["ls"])?.stdout;
    assert!(stdout.contains("httpbin/absolute"), "{}", stdout);
    assert!(stdout.contains("httpbin/relative"), "{}", stdout);

    Ok(())
}
