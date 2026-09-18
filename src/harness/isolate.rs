use std::ffi::OsStr;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Exit {
    Code(i32),
    Signal(i32),
    Timeout,
    Unknown,
}

impl Exit {
    pub fn is_success(&self) -> bool {
        matches!(self, Exit::Code(0))
    }
    pub fn is_crash(&self) -> bool {
        matches!(self, Exit::Signal(_))
    }
}

pub struct Isolated {
    pub exit: Exit,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_isolated<I, S>(program: &str, args: I) -> std::io::Result<Isolated>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(program).args(args).output()?;
    Ok(Isolated {
        exit: classify(&output.status),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

pub fn run_isolated_timeout<I, S>(
    program: &str,
    args: I,
    timeout: Duration,
) -> std::io::Result<Isolated>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let start = Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => {
                return Ok(Isolated {
                    exit: classify(&status),
                    stdout: drain_stdout(&mut child),
                    stderr: String::new(),
                });
            }
            None if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(Isolated {
                    exit: Exit::Timeout,
                    stdout: drain_stdout(&mut child),
                    stderr: String::new(),
                });
            }
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    }
}

fn drain_stdout(child: &mut std::process::Child) -> String {
    let mut buf = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut buf);
    }
    buf
}

#[cfg(unix)]
fn classify(status: &std::process::ExitStatus) -> Exit {
    use std::os::unix::process::ExitStatusExt;
    if let Some(code) = status.code() {
        Exit::Code(code)
    } else if let Some(sig) = status.signal() {
        Exit::Signal(sig)
    } else {
        Exit::Unknown
    }
}
