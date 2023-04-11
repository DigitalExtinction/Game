use std::{
    process::{Child, Command, Stdio},
    thread::sleep,
    time::Duration,
};

use assert_cmd::cargo::CommandCargoExt;
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};

pub fn spawn_and_wait() -> Child {
    let child = Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .stdin(Stdio::null())
        .spawn()
        .unwrap();
    sleep(Duration::from_millis(500));
    child
}

pub fn term_and_wait(mut child: Child) {
    let pid = Pid::from_raw(child.id().try_into().unwrap());
    kill(pid, Signal::SIGTERM).unwrap();
    child.wait().unwrap();
}
