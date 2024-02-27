use ls_proxy::entrypoint;
use std::{error::Error, path::PathBuf};
use tokio::process::Child;
use tokio_test::io::Mock;
use tokio_util::sync::CancellationToken;

pub struct TestApp {
    child: Child,
}

impl TestApp {
    pub(crate) async fn wait(&mut self) {
        let _ = self.child.wait().await;
    }
}

pub(crate) async fn spawn_app(
    image: String,
    code_path: &PathBuf,
    stdin: Mock,
    stdout: Mock,
    stderr: Mock,
) -> Result<TestApp, Box<dyn Error + 'static>> {
    let child = entrypoint::run(
        image,
        code_path.as_path(),
        stdin,
        stdout,
        stderr,
        CancellationToken::default(),
    )
    .await?;

    Ok(TestApp { child })
}
