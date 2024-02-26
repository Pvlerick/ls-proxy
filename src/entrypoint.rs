use std::{error::Error, fmt::Debug, path::Path, process::Stdio};

use crate::parser::MessageParser;

use tokio::{
    io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    process::{Child, Command},
};
use tokio_util::sync::CancellationToken;
use tracing::trace;

pub fn run<In, Out, Err>(
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

    println!("run called, about to start copy threads");

    start_copy_thread(
        stdin,
        child.stdin.take().expect("failed to get child stdin"),
        message_parser_inspector(),
        shutdown_token.clone(),
    );

    start_copy_thread(
        child.stdout.take().expect("failed to get child stdout"),
        stdout,
        message_parser_inspector(),
        shutdown_token.clone(),
    );

    start_copy_thread(
        child.stderr.take().expect("failed to get child stderr"),
        stderr,
        empty_inspector(),
        shutdown_token.clone(),
    );

    println!("returning child");

    Ok(child)
}

pub fn run_with_std(
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
}

fn start_copy_thread<'a, R, W, F: FnMut(&[u8])>(
    mut input: R,
    mut output: W,
    mut inspect_buffer: F,
    _shutdown_token: CancellationToken,
) where
    R: AsyncRead + std::marker::Unpin + Send + Debug + 'static,
    W: AsyncWrite + std::marker::Unpin + Send + Debug + 'static,
    F: Send + 'static,
{
    const BUFFER_SIZE: usize = 4 * 1024;

    trace!(
        "starting copy thread from {:?} to {:?} with buffer size {}",
        input,
        output,
        BUFFER_SIZE
    );

    println!("going to spawn thread");

    tokio::spawn(async move {
        println!("entering main loop");

        let mut buffer = [0u8; BUFFER_SIZE];

        loop {
            println!("main loop");

            match input.read(&mut buffer).await {
                Ok(0) => {}
                Ok(bytes_read) => {
                    let buf = &buffer[..bytes_read];
                    trace!("read {} bytes from {:?}", bytes_read, input);
                    trace!("[BUFFER] {}", String::from_utf8_lossy(&buf),);

                    inspect_buffer(&buf);

                    match output.write_all(&buf).await {
                        Err(e) => panic!("error: {:?}", e),
                        _ => {}
                    }

                    match output.flush().await {
                        Err(e) => panic!("error: {:?}", e),
                        _ => {}
                    }
                }
                Err(e) => panic!("error: {:?}", e),
            }
        }
    });
}

fn message_parser_inspector() -> impl FnMut(&[u8]) {
    let mut mp = MessageParser::new();

    move |buffer: &[u8]| {
        for msg in mp.parse(buffer) {
            // trace!("[MSG] {}", msg.payload);
            println!("[MSG] {}", msg.payload);
        }
    }
}

fn empty_inspector() -> impl FnMut(&[u8]) {
    |_: &[u8]| {}
}
