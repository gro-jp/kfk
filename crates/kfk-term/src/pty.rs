//! a program on a pseudo-terminal: the far end of a web terminal
use anyhow::{bail, Result};
use std::fs::File;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

pub struct Pty {
    master: File,
    child: Child,
}

fn winsize(cols: u16, rows: u16) -> libc::winsize {
    libc::winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 }
}

impl Pty {
    /// start `program args` with a new terminal of the given size as its controlling terminal
    pub fn spawn(program: &str, args: &[String], cols: u16, rows: u16) -> Result<Pty> {
        let (mut m, mut s) = (0, 0);
        let mut ws = winsize(cols, rows);
        if unsafe { libc::openpty(&mut m, &mut s, std::ptr::null_mut(), std::ptr::null_mut(), &mut ws) } != 0 {
            bail!("cannot open a terminal: {}", std::io::Error::last_os_error());
        }
        let (master, slave) = unsafe { (File::from_raw_fd(m), OwnedFd::from_raw_fd(s)) };
        unsafe { libc::fcntl(m, libc::F_SETFD, libc::FD_CLOEXEC) };
        let mut c = Command::new(program);
        c.args(args).env("TERM", "xterm-256color").env("COLORTERM", "truecolor").env_remove("TMUX");
        c.stdin(Stdio::from(slave.try_clone()?)).stdout(Stdio::from(slave.try_clone()?)).stderr(Stdio::from(slave));
        unsafe {
            c.pre_exec(|| {
                if libc::setsid() < 0 || libc::ioctl(0, libc::TIOCSCTTY as _, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = c.spawn().map_err(|e| anyhow::anyhow!("cannot run {program}: {e}"))?;
        Ok(Pty { master, child })
    }

    /// what the program prints; reading ends when the program does
    pub fn reader(&self) -> Result<File> {
        Ok(self.master.try_clone()?)
    }

    /// what is typed at the program
    pub fn writer(&self) -> Result<File> {
        Ok(self.master.try_clone()?)
    }

    pub fn resize(&self, cols: u16, rows: u16) {
        unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ as _, &winsize(cols, rows)) };
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
