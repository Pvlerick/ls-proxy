use ls_proxy::entrypoint;
use std::{
    env::temp_dir,
    error::Error,
    io::{Read, Stdin, Write},
    path::{Path, PathBuf},
    process::{ChildStdin, ChildStdout},
};
use tokio::{fs::create_dir, io::AsyncWrite, process::Child};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct TestApp {
    pub container_id: String,
    child: Child,
}

pub async fn spawn_app(image: &str, code_path: &Path) -> TestApp {
    let _proxy = entrypoint::run(image, code_path, CancellationToken::default())
        .expect("Failed to bin address or start server");

    TestApp {
        container_id: "".to_string(),
        child,
    }
}

impl AsyncWrite for TestApp {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
    }
}

pub async fn create_tmp_dir() -> PathBuf {
    let mut dir_path = temp_dir();
    dir_path.push(Uuid::new_v4().to_string());

    create_dir(&dir_path)
        .await
        .expect("failed to create tmp dir");

    dir_path
}
