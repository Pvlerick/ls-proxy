use std::{error::Error, path::PathBuf};

use clap::Parser;
use ls_proxy::entrypoint;
use ls_proxy::telemetry::{get_subscriber, init_subscriber};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = get_subscriber();
    init_subscriber(subscriber);

    debug!("ls-proxy started");

    let args = Args::parse();

    let shutdown_token = CancellationToken::new();

    let child =
        entrypoint::run_with_std(args.image, args.src_root_dir.as_path(), shutdown_token).await?;

    match child.wait_with_output().await {
        Ok(output) => {
            let child_exit_code = output
                .status
                .code()
                .expect("failed to get child process exit code");

            debug!("child process exit status code: {}", child_exit_code);

            if child_exit_code != 0 {
                warn!(
                    "child process terminated with exit status code {}",
                    child_exit_code
                );
                let remaining_stdout = String::from_utf8_lossy(&output.stdout);
                if !remaining_stdout.is_empty() {
                    warn!("child process remaining stdout: {}", remaining_stdout);
                }
                let remaining_stderr = String::from_utf8_lossy(&output.stderr);
                if !remaining_stderr.is_empty() {
                    warn!("child process remaining stderr: {}", remaining_stderr);
                }
            }
        }
        Err(e) => panic!("error: {:?}", e),
    }

    Ok(())
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
