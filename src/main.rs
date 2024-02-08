use std::{
    env,
    error::Error,
    fmt::Debug,
    io::{self, ErrorKind, Read, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
};

use ls_proxy::parser::MessageParser;

use tracing::{debug, trace};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

fn main() -> Result<(), Box<dyn Error>> {
    let _guard = set_tracing();

    debug!("ls-proxy started");

    let args: Vec<_> = env::args().collect();
    trace!("args {:?}\n", args);

    let mut tee_in = Command::new("tee")
        .stdout(Stdio::piped())
        .arg("/tmp/ls-proxy-in")
        .spawn()?;

    let mut tee_out = Command::new("tee")
        .stdin(Stdio::piped())
        .arg("/tmp/ls-proxy-out")
        .spawn()?;

    let mut child = Command::new("podman")
        .stdin(tee_in.stdout.take().expect("failed to get tee_in stdout"))
        .stdout(tee_out.stdin.take().expect("failed to get tee_out stdin"))
        // .stderr(Stdio::piped())
        .args([
            "run",
            "-i",
            "--rm",
            "-v",
            format!("{}:{}", &args[2], &args[2]).as_str(),
            &args[1],
        ])
        .spawn()?;

    // start_copy_thread(
    //     io::stdin(),
    //     child.stdin.take().expect("failed to get child stdin"),
    //     message_parser_inspector(),
    // );
    //
    // start_copy_thread(
    //     child.stdout.take().expect("failed to get child stdout"),
    //     io::stdout(),
    //     message_parser_inspector(),
    // );
    //
    // start_copy_thread(
    //     child.stderr.take().expect("failed to get child stderr"),
    //     io::stderr(),
    //     empty_inspector(),
    // );

    let child_exit_status = child
        .wait_with_output()
        .expect("failed waiting on child process termination");

    debug!(
        "child process exit status: {}",
        child_exit_status.status.code().unwrap()
    );

    Ok(())
}

fn start_copy_thread<'a, R, W, F: FnMut(&[u8])>(mut input: R, mut output: W, mut inspect_buffer: F)
where
    R: Read + Send + Debug + 'static,
    W: Write + Send + Debug + 'static,
    F: Send + 'static,
{
    const BUFFER_SIZE: usize = 8196;

    thread::spawn(move || {
        trace!(
            "starting copy thread from {:?} to {:?} with buffer size {}",
            input,
            output,
            BUFFER_SIZE
        );

        let mut buffer = [0u8; BUFFER_SIZE];

        loop {
            let bytes_read = input.read(&mut buffer).expect("failed to read");
            if bytes_read > 0 {
                trace!("read {} bytes from {:?}", bytes_read, input);
                trace!(
                    "[BUFFER] {}",
                    String::from_utf8_lossy(&buffer[..bytes_read])
                );

                inspect_buffer(&buffer[..bytes_read]);

                output
                    .write_all(&buffer[..bytes_read])
                    .expect("failed to write");
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

fn set_tracing() -> WorkerGuard {
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("log")
        .build(
            &Path::new(&env::var("HOME").expect("no HOME env variable found"))
                .join(".local/state/ls-proxy"),
        )
        .expect("failed to initialize file appender");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .with_env_var("LSPROXY_LOG")
                .from_env_lossy(),
        )
        .with_writer(non_blocking)
        .init();

    guard
}
