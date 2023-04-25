use std::{
    os::unix::process::CommandExt,
    process::{Child, Command, Stdio},
    thread::sleep,
    time::Duration,
};

use assert_cmd::cargo::CommandCargoExt;
use nix::{
    libc::{prctl, PR_SET_PDEATHSIG, SIGTERM},
    sys::signal::{kill, Signal},
    unistd::Pid,
};

pub fn spawn_and_wait() -> Child {
    let mut command = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

    unsafe {
        command.pre_exec(|| {
            let parent_pid = Pid::this();
            let result = prctl(PR_SET_PDEATHSIG, SIGTERM as usize, 0, 0, 0);
            if result != 0 || parent_pid != Pid::this() {
                panic!("Failed to set PR_SET_PDEATHSIG for the child process");
            }
            Ok(())
        });
    }

    let child = command.stdin(Stdio::null()).spawn().unwrap();
    sleep(Duration::from_millis(500));
    child
}

pub fn term_and_wait(mut child: Child) {
    let pid = Pid::from_raw(child.id().try_into().unwrap());
    kill(pid, Signal::SIGTERM).unwrap();
    child.wait().unwrap();
}
