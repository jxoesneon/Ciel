//! Subprocess execution with captured output and a timeout, mirroring
//! `subprocess.run(..., capture_output=True, text=True, timeout=...)` —
//! timeout kills the direct child only, like `Popen.kill()`.

use std::io::Read;
use std::process::{Command, ExitStatus, Stdio};
use std::time::Duration;

pub struct ProcOutput {
    pub status: Option<ExitStatus>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

impl ProcOutput {
    /// Python `CompletedProcess.returncode` — negative signal on unix.
    pub fn returncode(&self) -> i32 {
        match self.status {
            Some(s) => {
                if let Some(code) = s.code() {
                    code
                } else {
                    #[cfg(unix)]
                    {
                        use std::os::unix::process::ExitStatusExt;
                        s.signal().map(|sig| -sig).unwrap_or(-1)
                    }
                    #[cfg(not(unix))]
                    {
                        -1
                    }
                }
            }
            None => -1,
        }
    }
}

fn drain(
    child: &mut std::process::Child,
) -> (
    std::thread::JoinHandle<Vec<u8>>,
    std::thread::JoinHandle<Vec<u8>>,
) {
    let mut out_pipe = child.stdout.take().expect("stdout piped");
    let mut err_pipe = child.stderr.take().expect("stderr piped");
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
    (t_out, t_err)
}

fn finish(
    mut child: std::process::Child,
    t_out: std::thread::JoinHandle<Vec<u8>>,
    t_err: std::thread::JoinHandle<Vec<u8>>,
    status: Option<ExitStatus>,
    timed_out: bool,
) -> ProcOutput {
    let stdout = t_out.join().unwrap_or_default();
    let stderr = t_err.join().unwrap_or_default();
    // Reap in case timeout kill raced the status.
    let status = status.or_else(|| child.try_wait().ok().flatten());
    let _ = child; // child already waited via wait_timeout/wait paths
    ProcOutput {
        status,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        timed_out,
    }
}

/// Run `cmd` (already a Command with argv/env/cwd set) capturing stdout and
/// stderr; `timeout` None means no limit. On timeout the child is killed
/// and `timed_out` is set — mirrors subprocess.run's TimeoutExpired path.
pub fn run_cmd(cmd: &mut Command, timeout: Option<Duration>) -> Result<ProcOutput, std::io::Error> {
    use wait_timeout::ChildExt;
    let mut child = cmd.spawn()?;
    let (t_out, t_err) = drain(&mut child);
    let wait_res = match timeout {
        Some(t) => child.wait_timeout(t),
        None => child.wait().map(Some),
    };
    match wait_res {
        Ok(Some(st)) => Ok(finish(child, t_out, t_err, Some(st), false)),
        Ok(None) => {
            let _ = child.kill();
            let st = child.wait().ok();
            Ok(finish(child, t_out, t_err, st, true))
        }
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = t_out.join();
            let _ = t_err.join();
            Err(e)
        }
    }
}

/// `subprocess.run(cmd, shell=True)` → `/bin/sh -c cmd` on unix.
#[cfg(unix)]
pub fn run_shell(
    cmd_str: &str,
    cwd: &std::path::Path,
    env: &std::collections::HashMap<String, String>,
    timeout: Option<Duration>,
) -> Result<ProcOutput, std::io::Error> {
    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c").arg(cmd_str);
    cmd.current_dir(cwd);
    cmd.envs(env);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    run_cmd(&mut cmd, timeout)
}
