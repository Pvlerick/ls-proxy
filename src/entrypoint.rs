use std::{error::Error, fmt::Debug, path::Path, process::Stdio};

use crate::parser::MessageParser;

use tokio::{
    io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, Command},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::trace;

pub async fn run<In, Out, Err>(
    image: String,
    path: &Path,
    stdin: In,
    stdout: Out,
    stderr: Err,
    shutdown_token: CancellationToken,
) -> Result<Child, Box<dyn Error>>
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

    let _ = tokio::join!(
        start_copy_task(
            stdin,
            child.stdin.take().expect("failed to get child stdin"),
            message_parser_inspector(),
            shutdown_token.clone(),
        ),
        start_copy_task(
            child.stdout.take().expect("failed to get child stdout"),
            stdout,
            message_parser_inspector(),
            shutdown_token.clone(),
        ),
        start_copy_task(
            child.stderr.take().expect("failed to get child stderr"),
            stderr,
            empty_inspector(),
            shutdown_token.clone(),
        ),
    );

    Ok(child)
}

pub async fn run_with_std(
    image: String,
    path: &Path,
    shutdown_token: CancellationToken,
) -> Result<Child, Box<dyn Error>> {
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

async fn start_copy_task<'a, R, W, F: FnMut(&[u8])>(
    input: R,
    output: W,
    mut inspect_buffer: F,
    _shutdown_token: CancellationToken,
) -> JoinHandle<()>
where
    R: AsyncRead + std::marker::Unpin + Send + Debug + 'static,
    W: AsyncWrite + std::marker::Unpin + Send + Debug + 'static,
    F: Send + 'static,
{
    const BUFFER_SIZE: usize = 8 * 1024;

    trace!(
        "starting copy thread from {:?} to {:?} with buffer size {}",
        input,
        output,
        BUFFER_SIZE
    );

    tokio::spawn(async move {
        let mut buffer = [0u8; BUFFER_SIZE];

        let mut reader = BufReader::new(input);
        let mut writer = BufWriter::new(output);

        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => {}
                Ok(bytes_read) => {
                    let mut read_slice = &buffer[..bytes_read];
                    trace!("read {} bytes from {:?}", bytes_read, reader);
                    trace!("[BUFFER] {}", String::from_utf8_lossy(&read_slice),);

                    inspect_buffer(&read_slice);

                    trace!("inspection done, writing to output...");

                    match writer.write_all_buf(&mut read_slice).await {
                        Err(e) => panic!("failed to write to {:?}: {:?}", writer, e),
                        _ => {}
                    }

                    trace!("flushing...");

                    match writer.flush().await {
                        Err(e) => panic!("failed to flush {:?}: {:?}", writer, e),
                        _ => {}
                    }

                    trace!("done copying");
                }
                Err(e) => panic!("error: {:?}", e),
            }
        }
    })
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
