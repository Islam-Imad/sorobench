use std::ffi::OsStr;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const TIMEOUT: Duration = Duration::from_secs(10);

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
        .stderr(Stdio::piped())
        .spawn()?;

    let start = Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => {
                return Ok(Isolated {
                    exit: classify(&status),
                    stdout: drain(child.stdout.take()),
                    stderr: drain(child.stderr.take()),
                });
            }
            None if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(Isolated {
                    exit: Exit::Timeout,
                    stdout: drain(child.stdout.take()),
                    stderr: drain(child.stderr.take()),
                });
            }
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    }
}

/// Read a child pipe to end-of-stream, swallowing I/O errors (best-effort).
fn drain<R: Read>(stream: Option<R>) -> String {
    let mut buf = String::new();
    if let Some(mut s) = stream {
        let _ = s.read_to_string(&mut buf);
    }
    buf
}

pub fn panic_message(stderr: &str) -> Option<String> {
    let mut lines = stderr.lines();
    while let Some(line) = lines.next() {
        if line.contains("panicked at") {
            let mut body: Vec<&str> = Vec::new();
            for l in lines.by_ref() {
                let t = l.trim_start();
                if t.starts_with("note:") || t == "stack backtrace:" {
                    break;
                }
                body.push(l.trim_end());
            }
            // Drop blank lines at the edges, then join the message verbatim.
            while body.first().is_some_and(|l| l.trim().is_empty()) {
                body.remove(0);
            }
            while body.last().is_some_and(|l| l.trim().is_empty()) {
                body.pop();
            }
            let joined = body.join("\n");
            let joined = joined.trim();
            if !joined.is_empty() {
                return Some(joined.to_string());
            }
            return Some(line.trim().to_string());
        }
    }
    stderr
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with("note:"))
        .map(String::from)
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
