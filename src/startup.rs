use std::{
    error::Error,
    fmt::Debug,
    io::{self, Read, Write},
    path::Path,
    process::{Child, Command, Stdio},
};

use crate::parser::MessageParser;

use tokio_util::sync::CancellationToken;
use tracing::trace;

pub fn run(
    image: &str,
    path: &Path,
    shutdown_token: CancellationToken,
) -> Result<Child, Box<dyn Error>> {
    let path = path.to_str().expect("failed to convert &Path to &str");

    let mut child = Command::new("podman")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args([
            "run",
            "-i",
            "--rm",
            "-v",
            format!("{}:{}", path, path).as_str(),
            image,
        ])
        .spawn()?;

    start_copy_thread(
        io::stdin(),
        child.stdin.take().expect("failed to get child stdin"),
        message_parser_inspector(),
        shutdown_token.clone(),
    );

    start_copy_thread(
        child.stdout.take().expect("failed to get child stdout"),
        io::stdout(),
        message_parser_inspector(),
        shutdown_token.clone(),
    );

    start_copy_thread(
        child.stderr.take().expect("failed to get child stderr"),
        io::stderr(),
        empty_inspector(),
        shutdown_token.clone(),
    );

    Ok(child)
}

fn start_copy_thread<'a, R, W, F: FnMut(&[u8])>(
    mut input: R,
    mut output: W,
    mut inspect_buffer: F,
    shutdown_token: CancellationToken,
) where
    R: Read + Send + Debug + 'static,
    W: Write + Send + Debug + 'static,
    F: Send + 'static,
{
    const BUFFER_SIZE: usize = 4 * 1024;

    trace!(
        "starting copy thread from {:?} to {:?} with buffer size {}",
        input,
        output,
        BUFFER_SIZE
    );

    let _ = tokio::spawn(async move {
        let mut buffer = [0u8; BUFFER_SIZE];

        tokio::select! {
            _ = async {
                loop {
                    let bytes_read = input.read(&mut buffer).expect("failed to read");
                    let buf = &buffer[..bytes_read];
                    if bytes_read > 0 {
                        trace!("read {} bytes from {:?}", bytes_read, input);
                        trace!("[BUFFER] {}", String::from_utf8_lossy(&buf),);

                        inspect_buffer(&buf);

                        output.write_all(&buf).expect("failed to write");
                        output.flush().expect("failed to flush output");
                    }
                }
            } => {}
            _ = shutdown_token.cancelled() => {
                trace!("shutdown requested, stop copying from {:?} to {:?}", input, output);
            }
        }
    });
}

fn message_parser_inspector() -> impl FnMut(&[u8]) {
    let mut mp = MessageParser::new();

    move |buffer: &[u8]| {
        for msg in mp.parse(buffer) {
            trace!("[MSG] {}", msg.payload);
        }
    }
}

fn empty_inspector() -> impl FnMut(&[u8]) {
    |_: &[u8]| {}
}
