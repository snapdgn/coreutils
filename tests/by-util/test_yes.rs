use std::io::Read;
use std::process::ExitStatus;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

use crate::common::util::*;

#[cfg(unix)]
fn check_termination(result: &ExitStatus) {
    assert_eq!(result.signal(), Some(libc::SIGPIPE as i32));
}

#[cfg(not(unix))]
fn check_termination(result: &ExitStatus) {
    assert!(result.success(), "yes did not exit successfully");
}

/// Run `yes`, capture some of the output, close the pipe, and verify it.
fn run(args: &[&str], expected: &[u8]) {
    let mut cmd = new_ucmd!();
    let mut child = cmd.args(args).run_no_wait();
    let mut stdout = child.stdout.take().unwrap();
    let mut buf = vec![0; expected.len()];
    stdout.read_exact(&mut buf).unwrap();
    drop(stdout);
    check_termination(&child.wait().unwrap());
    assert_eq!(buf.as_slice(), expected);
}

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_simple() {
    run(&[], b"y\ny\ny\ny\n");
}

#[test]
fn test_args() {
    run(&["a", "bar", "c"], b"a bar c\na bar c\na ba");
}

#[test]
fn test_long_output() {
    run(&[], "y\n".repeat(512 * 1024).as_bytes());
}

/// Test with an output that seems likely to get mangled in case of incomplete writes.
#[test]
fn test_long_odd_output() {
    run(&["abcdef"], "abcdef\n".repeat(1024 * 1024).as_bytes());
}

/// Test with an input that doesn't fit in the standard buffer.
#[test]
fn test_long_input() {
    #[cfg(not(windows))]
    const TIMES: usize = 14000;
    // On Windows the command line is limited to 8191 bytes.
    // This is not actually enough to fill the buffer, but it's still nice to
    // try something long.
    #[cfg(windows)]
    const TIMES: usize = 500;
    let arg = "abcdef".repeat(TIMES) + "\n";
    let expected_out = arg.repeat(30);
    run(&[&arg[..arg.len() - 1]], expected_out.as_bytes());
}

#[test]
#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd"))]
fn test_piped_to_dev_full() {
    use std::fs::OpenOptions;

    for append in [true, false] {
        {
            let dev_full = OpenOptions::new()
                .write(true)
                .append(append)
                .open("/dev/full")
                .unwrap();

            new_ucmd!()
                .set_stdout(dev_full)
                .fails()
                .stderr_contains("No space left on device");
        }
    }
}
