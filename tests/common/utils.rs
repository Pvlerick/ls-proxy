use std::{env, path::PathBuf};
use tokio::fs;
use uuid::Uuid;

pub async fn create_tmp_dir() -> PathBuf {
    let mut dir_path = env::temp_dir();
    dir_path.push(Uuid::new_v4().to_string());

    fs::create_dir(&dir_path)
        .await
        .expect("failed to create tmp dir");

    dir_path
}
