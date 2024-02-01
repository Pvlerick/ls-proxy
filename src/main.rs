use std::{
    env,
    error::Error,
    io::{self, Read, Write},
    path::Path,
    process::{exit, Command, Stdio},
    thread,
};

use signal_hook::{consts::SIGTERM, iterator::Signals};
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
    loop {
        let n = io::stdin().read(&mut buffer[..])?;
        child_stdin.write(&buffer[..n])?;
    }
    //

    //
    // loop {
    //     if io::stdin().read_line(&mut buff)? > 0 {
    //         trace!(buff);
    //         buff.clear();
    //     }
    // }

    // let mut tee = Command::new("tee")
    //     .arg("/tmp/ls-proxy.log")
    //     .stdout(Stdio::piped())
    //     .spawn()?;
    // let tee_stdout = tee.stdout.take().expect("failed to get tee stdout");
    //
    // tee.wait()?;
    // podman.wait()?;
    //
    // Ok(())
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
