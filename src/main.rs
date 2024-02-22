use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use ls_proxy::entrypoint;
use tokio_util::sync::CancellationToken;
use tracing::{debug, trace, warn};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _guard = set_tracing();

    debug!("ls-proxy started");

    let args = Args::parse();

    let shutdown_token = CancellationToken::new();

    // tokio::select! {
    //     result = startup::run(args[1], args[2], shutdown_token) => {
    //         result
    //     }
    //     termination = unix::signal(SignalKind::terminate()) => {
    //         debug!("received shutdown signal");
    //         shutdown_token.cancel();
    //         Ok(())
    //     }
    // }
    let child = entrypoint::run_with_std(&args.image, args.src_root_dir.as_path(), shutdown_token)?;

    let child_output = child
        .wait_with_output()
        .expect("failed waiting on child process termination");

    let child_exit_code = child_output
        .status
        .code()
        .expect("failed to get child process exit code");

    debug!("child process exit status code: {}", child_exit_code);

    if child_exit_code != 0 {
        warn!(
            "child process terminated with exit status code {}",
            child_exit_code
        );
        let remaining_stdout = String::from_utf8_lossy(&child_output.stdout);
        if !remaining_stdout.is_empty() {
            warn!("child process remaining stdout: {}", remaining_stdout);
        }
        let remaining_stderr = String::from_utf8_lossy(&child_output.stderr);
        if !remaining_stderr.is_empty() {
            warn!("child process remaining stderr: {}", remaining_stderr);
        }
    }

    Ok(())
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
                .with_default_directive(LevelFilter::DEBUG.into())
                .with_env_var("LSPROXY_LOG")
                .from_env_lossy(),
        )
        .with_writer(non_blocking)
        .init();

    guard
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Image
    #[arg()]
    image: String,
    /// Source Root Directory
    #[arg()]
    src_root_dir: PathBuf,
}
