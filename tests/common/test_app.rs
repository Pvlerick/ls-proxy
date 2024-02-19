use std::{
    env::temp_dir,
    path::{Path, PathBuf},
};
use tokio::fs::create_dir;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct TestApp {
    pub container_id: String,
}

pub async fn spawn_app(image: &str, code_path: &Path) -> TestApp {
    let _proxy = ls_proxy::startup::run(image, code_path, CancellationToken::default())
        .expect("Failed to bin address or start server");

    TestApp {
        container_id: "".to_string(),
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
