use std::{error::Error, fmt::Debug, path::PathBuf, process::Stdio};

use crate::parser::MessageParser;

use tokio::{
    io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    process::Command,
    task::{JoinHandle, JoinSet},
};
use tokio_util::sync::CancellationToken;
use tracing::trace;

pub async fn run<In, Out, Err>(
    image: String,
    path: PathBuf,
    stdin: In,
    stdout: Out,
    stderr: Err,
    shutdown_token: CancellationToken,
) -> Result<JoinSet<()>, Box<dyn Error + Send + Sync>>
where
    In: AsyncRead + std::marker::Unpin + Send + Debug + 'static,
    Out: AsyncWrite + std::marker::Unpin + Send + Debug + 'static,
    Err: AsyncWrite + std::marker::Unpin + Send + Debug + 'static,
{
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
            image.as_str(),
        ])
        .spawn()?;

    let mut tasks = JoinSet::new();

    tasks.spawn(async move {
        tokio::select! {
            _ = start_copy_loop(
                stdin,
                child.stdin.take().expect("failed to get child stdin"),
                message_parser_inspector()) => {}
            _ = start_copy_loop(
                child.stdout.take().expect("failed to get child stdout"),
                stdout,
                message_parser_inspector()) => {}
            _ = start_copy_loop(
                child.stderr.take().expect("failed to get child stderr"),
                stderr,
                empty_inspector()) => {}
            _ = shutdown_token.cancelled() => {}
        }
    });

    Ok(tasks)
}

pub async fn run_with_std(
    image: String,
    path: PathBuf,
    shutdown_token: CancellationToken,
) -> Result<JoinSet<()>, Box<dyn Error + Send + Sync>> {
    run(
        image,
        path,
        io::stdin(),
        io::stdout(),
        io::stderr(),
        shutdown_token,
    )
    .await
}

#[allow(unreachable_code)]
async fn start_copy_loop<'a, R, W, F: FnMut(&[u8])>(
    mut input: R,
    mut output: W,
    mut inspect_buffer: F,
) -> Result<(), io::Error>
where
    R: AsyncRead + std::marker::Unpin + Send + Debug + 'static,
    W: AsyncWrite + std::marker::Unpin + Send + Debug + 'static,
    F: Send + 'static,
{
    const BUFFER_SIZE: usize = 8 * 1024;
    let mut buffer = [0u8; BUFFER_SIZE];

    trace!(
        "starting copy loop from {:?} to {:?} with buffer size {}",
        input,
        output,
        BUFFER_SIZE
    );

    loop {
        let bytes_read = input.read(&mut buffer).await?;
        if bytes_read > 0 {
            let mut read_slice = &buffer[..bytes_read];
            trace!("read {} bytes from {:?}", bytes_read, input);
            trace!("[BUFFER] {}", String::from_utf8_lossy(&read_slice));

            inspect_buffer(&read_slice);

            trace!("inspection done, writing to output...");

            output.write_all(&mut read_slice).await?;

            trace!("flushing...");

            output.flush().await?;

            trace!("done copying");
        }
    }

    // Ok::<_, io::Error>(())
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
