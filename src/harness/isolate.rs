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

    // Drain both pipes on their own threads so a chatty child can never block
    // on a full pipe (which would look like a timeout).
    let out = spawn_reader(child.stdout.take());
    let err = spawn_reader(child.stderr.take());

    let start = Instant::now();
    let exit = loop {
        match child.try_wait()? {
            Some(status) => break classify(&status),
            None if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                break Exit::Timeout;
            }
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    };
    Ok(Isolated {
        exit,
        stdout: out.join().unwrap_or_default(),
        stderr: err.join().unwrap_or_default(),
    })
}

fn spawn_reader<R: Read + Send + 'static>(pipe: Option<R>) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut p) = pipe {
            let _ = p.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

/// The informative part of a crashed child's stderr, on one line: the Rust
/// panic (location + full message, see [`panic_message`]), an LLVM assertion,
/// or a stack overflow; otherwise the last few lines.
pub fn crash_summary(stderr: &str) -> String {
    let lines: Vec<&str> = stderr
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let mut picked: Vec<String> = Vec::new();
    for l in lines.iter() {
        if l.contains("panicked at") {
            // The `panicked at <location>` line, then the full message body.
            picked.push(l.to_string());
            if let Some(body) = panic_message(stderr) {
                picked.extend(body.lines().map(|b| b.trim().to_string()));
            }
            break;
        }
        if l.contains("Assertion")
            || l.contains("overflowed its stack")
            || l.starts_with("LLVM ERROR")
        {
            picked.push(l.to_string());
            break;
        }
    }
    if picked.is_empty() {
        let n = lines.len();
        picked = lines[n.saturating_sub(3)..]
            .iter()
            .map(|l| l.to_string())
            .collect();
    }
    let joined = picked.join(" | ");
    let joined: String = if joined.chars().count() > 2000 {
        let cut: String = joined.chars().take(2000).collect();
        format!("{cut}…")
    } else {
        joined
    };
    joined
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
