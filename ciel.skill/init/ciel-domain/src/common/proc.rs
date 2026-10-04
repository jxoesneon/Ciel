//! Subprocess execution with capture + timeout + process-group termination,
//! mirroring `subprocess.Popen(..., start_new_session=True)` +
//! `os.killpg(SIGTERM)` semantics from the Python studio executor.

use std::io::Read;
use std::process::{Command, ExitStatus, Stdio};
use std::time::Duration;

pub struct ProcOutput {
    pub status: Option<ExitStatus>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

/// Run `argv` capturing stdout/stderr; on timeout, SIGTERM the child's
/// process group. `timeout_secs <= 0` means no limit.
#[cfg(unix)]
pub fn run_capture(argv: &[String], timeout_secs: u64) -> Result<ProcOutput, std::io::Error> {
    use std::os::unix::process::CommandExt;
    use wait_timeout::ChildExt;

    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0); // start_new_session equivalent (new pgid)

    let mut child = cmd.spawn()?;

    let mut out_pipe = child.stdout.take().unwrap();
    let mut err_pipe = child.stderr.take().unwrap();
    let t_out = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = out_pipe.read_to_end(&mut buf);
        buf
    });
    let t_err = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = err_pipe.read_to_end(&mut buf);
        buf
    });

    let wait_res = if timeout_secs > 0 {
        child.wait_timeout(Duration::from_secs(timeout_secs))
    } else {
        child.wait().map(Some)
    };

    let (status, timed_out) = match wait_res {
        Ok(Some(st)) => (Some(st), false),
        Ok(None) => {
            // Timed out — kill the whole process group like os.killpg(SIGTERM)
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGTERM);
            }
            // Brief grace then SIGKILL if still alive.
            std::thread::sleep(Duration::from_millis(500));
            match child.try_wait() {
                Ok(Some(st)) => (Some(st), true),
                _ => {
                    unsafe {
                        libc::kill(-(child.id() as i32), libc::SIGKILL);
                    }
                    let _ = child.wait();
                    (None, true)
                }
            }
        }
        Err(e) => {
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.wait();
            return Err(e);
        }
    };

    let stdout = t_out.join().unwrap_or_default();
    let stderr = t_err.join().unwrap_or_default();
    Ok(ProcOutput {
        status,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        timed_out,
    })
}

#[cfg(not(unix))]
pub fn run_capture(argv: &[String], timeout_secs: u64) -> Result<ProcOutput, std::io::Error> {
    use wait_timeout::ChildExt;
    let mut child = Command::new(&argv[0])
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut out_pipe = child.stdout.take().unwrap();
    let mut err_pipe = child.stderr.take().unwrap();
    let t_out = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = out_pipe.read_to_end(&mut buf);
        buf
    });
    let t_err = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = err_pipe.read_to_end(&mut buf);
        buf
    });
    let wait_res = if timeout_secs > 0 {
        child.wait_timeout(Duration::from_secs(timeout_secs))
    } else {
        child.wait().map(Some)
    };
    let (status, timed_out) = match wait_res {
        Ok(Some(st)) => (Some(st), false),
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            (None, true)
        }
        Err(e) => return Err(e),
    };
    let stdout = t_out.join().unwrap_or_default();
    let stderr = t_err.join().unwrap_or_default();
    Ok(ProcOutput {
        status,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        timed_out,
    })
}
