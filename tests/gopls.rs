use tokio::{fs::File, io::AsyncWriteExt};

use crate::common::create_tmp_dir;
use crate::common::spawn_app;

mod common;

#[tokio::test]
async fn gopls() {
    let main_dir_path = create_tmp_dir().await;
    let main_file_path = main_dir_path.join("main.go");

    let mut main_file = File::create(&main_file_path).await.unwrap();
    main_file
        .write(
            r#"package main

import "fmt"

func main() {
	fmt.Println("Hello World!")
}"#
            .as_bytes(),
        )
        .await
        .unwrap();

    let _app = spawn_app("quay.io/pvlerick/gopls:0.14.2-r0", main_dir_path.as_path());

    assert_eq!("", main_dir_path.to_str().unwrap());
    assert_eq!("", main_file_path.to_str().unwrap());
}
