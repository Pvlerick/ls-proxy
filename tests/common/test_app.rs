use ls_proxy::entrypoint;
use std::{error::Error, path::PathBuf};
use tokio::process::Child;
use tokio_test::io::{Builder, Handle};
use tokio_util::sync::CancellationToken;

pub struct TestApp {
    _child: Child,
    stdin: Handle,
    stdout: Handle,
    stderr: Handle,
}

impl TestApp {
    pub(crate) fn write_stdin(&mut self, payload: &[u8]) {
        self.stdin.write(payload);
    }

    pub(crate) fn read_stdout(&mut self) -> Option<Vec<u8>> {
        let mut buf = Vec::new();
        self.stdout.read(&mut buf);
        if buf.len() > 0 {
            return Some(buf);
        } else {
            return None;
        }
    }

    pub(crate) fn read_stderr(&mut self) -> Option<Vec<u8>> {
        let mut buf = Vec::new();
        self.stderr.read(&mut buf);
        if buf.len() > 0 {
            return Some(buf);
        } else {
            return None;
        }
    }
}

pub(crate) fn spawn_app(
    image: String,
    code_path: PathBuf,
) -> Result<TestApp, Box<dyn Error + 'static>> {
    let (stdin_mock, stdin_handle) = Builder::new().write(b"hello gopls").build_with_handle();
    let (stdout_mock, stdout_handle) = Builder::new().build_with_handle();
    let (stderr_mock, stderr_handle) = Builder::new().build_with_handle();

    let child = entrypoint::run(
        image,
        code_path.as_path(),
        stdin_mock,
        stdout_mock,
        stderr_mock,
        CancellationToken::default(),
    )?;

    Ok(TestApp {
        _child: child,
        stdin: stdin_handle,
        stdout: stdout_handle,
        stderr: stderr_handle,
    })
}
