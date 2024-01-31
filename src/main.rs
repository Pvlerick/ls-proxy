use std::{
    env,
    error::Error,
    io::{stdin, Write},
    path::Path,
    process::{exit, Command, Stdio},
    thread,
};

use signal_hook::{consts::SIGTERM, iterator::Signals};
use tracing::{debug, info, trace};
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
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("LSPROXY_LOG")
                .from_env_lossy(),
        )
        .with_writer(non_blocking)
        .init();

    set_signals_handler()?;

    debug!("proxy started");

    let args: Vec<_> = env::args().collect();
    trace!("args {:?}\n", args);

    let mut child = Command::new("gopls")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let (Some(mut child_stdin), Some(mut child_stdout)) =
        (child.stdin.take(), child.stdout.take())
    {
        debug!("child stdin retreived");
        loop {
            let stdin = stdin();
            let mut buff = String::new();
            if stdin.read_line(&mut buff)? > 0 {
                info!(buff);
                child_stdin.write_all(buff.as_str().as_bytes())?;
            }
        }
    }

    Ok(())
}

fn set_signals_handler() -> Result<(), Box<dyn Error>> {
    debug!("setting signals handler");

    let mut signals = Signals::new(&[SIGTERM])?;
    let _ = signals.handle();

    thread::spawn(move || {
        for sig in signals.forever() {
            debug!("Received signal '{}', shutting down...", sig);
            exit(0);
        }
    });

    Ok(())
}
