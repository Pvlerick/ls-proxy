use ls_proxy::entrypoint;
use std::{error::Error, path::PathBuf};
use tokio::process::Child;
use tokio_util::sync::CancellationToken;

use super::{queued_reader::QueuedReader, queued_writer::QueuedWriter};

pub struct TestApp<'a> {
    _child: Child,
    stdin: QueuedReader<'a>,
    stdout: QueuedWriter,
    stderr: QueuedWriter,
}

impl<'a> TestApp<'a> {
    pub(crate) fn write_stdin(&mut self, payload: &'a [u8]) {
        self.stdin.write(payload);
    }

    pub(crate) fn read_stdout(&mut self) -> Option<Vec<u8>> {
        self.stdout.read()
    }

    pub(crate) fn read_stderr(&mut self) -> Option<Vec<u8>> {
        self.stderr.read()
    }
}

pub(crate) fn spawn_app<'a>(
    image: String,
    code_path: PathBuf,
) -> Result<TestApp<'static>, Box<dyn Error + 'static>> {
    let stdin = QueuedReader::<'static>::new();
    let stdout = QueuedWriter::new();
    let stderr = QueuedWriter::new();

    let child = entrypoint::run(
        image,
        code_path.as_path(),
        stdin.clone(),
        stdout.clone(),
        stderr.clone(),
        CancellationToken::default(),
    )?;

    Ok(TestApp {
        _child: child,
        stdin,
        stdout,
        stderr,
    })
}
