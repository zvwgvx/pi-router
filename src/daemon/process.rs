use crate::error::RouterError;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tracing::{info, warn, error};

/// A supervised OS process. Tracks the `Child` handle and restart state.
pub struct ManagedProcess {
    pub name: String,
    prog: String,
    args: Vec<String>,
    child: Option<Child>,
    pub restart_count: u32,
}

impl ManagedProcess {
    /// Spawn the process immediately.
    pub fn spawn(
        name: impl Into<String>,
        prog: impl Into<String>,
        args: Vec<String>,
    ) -> Result<Self, RouterError> {
        let name = name.into();
        let prog = prog.into();

        let child = start_process(&name, &prog, &args)?;
        Ok(Self {
            name,
            prog,
            args,
            child: Some(child),
            restart_count: 0,
        })
    }

    /// Check whether the child process is still running.
    /// Calls `try_wait` to reap a zombie if it has exited.
    pub fn is_alive(&mut self) -> bool {
        match &mut self.child {
            None => false,
            Some(child) => match child.try_wait() {
                Ok(None) => true,          // still running
                Ok(Some(status)) => {
                    warn!(name = %self.name, ?status, "Process exited");
                    false
                }
                Err(e) => {
                    error!(name = %self.name, err = %e, "try_wait failed");
                    false
                }
            },
        }
    }

    /// Kill the current child (SIGTERM → wait briefly → SIGKILL) and restart.
    pub fn restart(&mut self) -> Result<(), RouterError> {
        self.restart_count += 1;
        warn!(name = %self.name, count = self.restart_count, "Restarting process");
        self.kill_child();
        let child = start_process(&self.name, &self.prog, &self.args)?;
        self.child = Some(child);
        Ok(())
    }

    /// Gracefully terminate the process.
    pub fn stop(&mut self) {
        info!(name = %self.name, "Stopping process");
        self.kill_child();
    }

    // ─── Internal ────────────────────────────────────────────────────────────

    fn kill_child(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Send SIGTERM via kill(2) on Unix
            #[cfg(unix)]
            {
                let pid = child.id() as i32;
                libc_kill(pid, libc::SIGTERM as i32);
            }

            // Give it a moment to clean up, then force kill if needed
            let deadline = std::time::Instant::now() + Duration::from_secs(3);
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    _ => {}
                }
                if std::time::Instant::now() >= deadline {
                    warn!(name = %self.name, "Process did not exit in time — sending SIGKILL");
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        self.kill_child();
    }
}

// ─── SIGTERM helper ──────────────────────────────────────────────────────────

#[cfg(unix)]
fn libc_kill(pid: i32, sig: i32) {
    unsafe { libc::kill(pid as libc::pid_t, sig as libc::c_int); }
}

#[cfg(not(unix))]
fn libc_kill(_pid: i32, _sig: i32) {}

// ─── Helper ───────────────────────────────────────────────────────────────────

#[cfg(not(target_os = "macos"))]
fn start_process(name: &str, prog: &str, args: &[String]) -> Result<Child, RouterError> {
    info!(name, prog, ?args, "Spawning process");
    Command::new(prog)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| RouterError::Daemon(format!("cannot spawn `{prog}`: {e}")))
}

#[cfg(target_os = "macos")]
fn start_process(name: &str, prog: &str, _args: &[String]) -> Result<Child, RouterError> {
    tracing::warn!("MacOS detected. Mocking process {name} ({prog}) with sleep");
    Command::new("sleep")
        .arg("86400")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| RouterError::Daemon(format!("cannot spawn mock sleep: {e}")))
}
