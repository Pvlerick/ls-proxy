use std::{env, error::Error, io::stdin, path::Path, process::exit, thread};

use ls_proxy::info;
use signal_hook::{consts::SIGTERM, iterator::Signals};
use tracing::{event, span, Level};
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

    let span = span!(Level::TRACE, "main");
    let _ = span.enter();

    event!(Level::DEBUG, "proxy started");

    let mut signals = Signals::new(&[SIGTERM])?;

    let _ = signals.handle();
    thread::spawn(move || {
        for sig in signals.forever() {
            event!(Level::DEBUG, "Received signal '{}', shutting down...", sig);
            exit(0);
        }
    });

    let args: Vec<_> = env::args().collect();
    event!(Level::DEBUG, "args: {:?}\n", args);

    loop {
        let mut buff = String::new();
        let stdin = stdin();
        stdin.read_line(&mut buff).expect("read from stdin failed");
        if buff.len() > 0 {
            match process_message(buff) {
                Err(e) => return Err(e),
                _ => (),
            }
        }
    }
}

fn process_message(msg: String) -> Result<(), Box<dyn Error>> {
    let span = span!(Level::TRACE, "process_message");
    let _ = span.enter();

    info!(msg);
    info!("format: {}", "erfg");

    Ok(())
}
