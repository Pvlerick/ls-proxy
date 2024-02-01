use std::{
    env,
    error::Error,
    io::{self, Read, Write},
    path::Path,
    process::{exit, Command, Stdio},
    thread,
};

use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use tracing::{debug, trace};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

fn main() -> Result<(), Box<dyn Error>> {
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("log")
        .build(
            &Path::new(&env::var("HOME").expect("no HOME env variable found"))
                .join(".local/state/ls-proxy"),
        )
        .expect("failed to initialize file appender");

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .with_env_var("LSPROXY_LOG")
                .from_env_lossy(),
        )
        .with_writer(non_blocking)
        .init();

    set_signals_handler()?;

    debug!("proxy started");

    let args: Vec<_> = env::args().collect();
    trace!("args {:?}\n", args);

    let mut child = Command::new("podman")
        .stdin(Stdio::piped())
        .args(["run", "-i", "--rm", "-v", "/tmp:/tmp", "gopls"])
        .spawn()?;

    let mut buffer = [0u8; 1024];
    let mut child_stdin = child.stdin.take().expect("failed to get child stdin");

    thread::spawn(move || loop {
        let n = io::stdin()
            .read(&mut buffer[..])
            .expect("failed to read from stdin");
        if n > 0 {
            trace!("read {} bytes from parent stdin", n);
            let msg = String::from_utf8_lossy(&buffer[..n]);
            trace!("{}", msg);
            child_stdin
                .write(&buffer[..n])
                .expect("failed to write to child stdin");
        }
    });

    child.wait()?;

    Ok(())
}

fn set_signals_handler() -> Result<(), Box<dyn Error>> {
    debug!("setting up signals handler");

    let mut signals = Signals::new(&[SIGTERM, SIGINT])?;
    let _ = signals.handle();

    thread::spawn(move || {
        for sig in signals.forever() {
            debug!("Received signal '{}', shutting down...", sig);
            exit(0);
        }
    });

    Ok(())
}
