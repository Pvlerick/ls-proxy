use ls_proxy::entrypoint;
use std::{error::Error, path::Path};
use tokio::process::Child;
use tokio_util::sync::CancellationToken;

use super::{queued_reader::QueuedReader, queued_writer::QueuedWriter};

pub struct TestApp<'a> {
    child: Child,
    stdin: QueuedReader<'a>,
    stdout: QueuedWriter,
    stderr: QueuedWriter,
}

impl<'a> TestApp<'a> {
    pub(crate) fn write_stdin(&mut self, payload: &'a str) {
        self.stdin.write(payload.as_bytes());
    }
}

pub(crate) fn spawn_app<'a>(
    image: &str,
    code_path: &Path,
) -> Result<TestApp<'a>, Box<dyn Error + 'a>> {
    let stdin = QueuedReader::<'a>::new();
    let stdout = QueuedWriter::new();
    let stderr = QueuedWriter::new();

    let child = entrypoint::run(
        image.clone(),
        code_path.clone(),
        stdin.clone(),
        stdout.clone(),
        stderr.clone(),
        CancellationToken::default(),
    )?;

    Ok(TestApp {
        child,
        stdin,
        stdout,
        stderr,
    })
}
