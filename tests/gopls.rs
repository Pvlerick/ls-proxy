use once_cell::sync::Lazy;
use std::{error::Error, io};
use tracing::info;

use ls_proxy::{entrypoint, telemetry::init_subscriber};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tokio_test::io::Builder;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

use crate::common::create_tmp_dir;

mod common;

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .with_env_var("LSPROXY_LOG")
                .from_env_lossy(),
        )
        .with_writer(io::stdout)
        .finish();

    init_subscriber(subscriber);
});

#[tokio::test]
async fn gopls() -> Result<(), Box<dyn Error + Send + Sync>> {
    Lazy::force(&TRACING);

    let main_dir_path = create_tmp_dir().await;
    let main_file_path = main_dir_path.join("main.go");

    info!("test staring");

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

    // let stdin = Builder::new().read(br#"Content-Length: 3111\r\n{"params":{"trace":"off","rootUri":null,"capabilities":{"textDocument":{"documentHighlight":{"dynamicRegistration":false},"documentSymbol":{"dynamicRegistration":false,"symbolKind":{"valueSet":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26]},"hierarchicalDocumentSymbolSupport":true},"callHierarchy":{"dynamicRegistration":false},"hover":{"dynamicRegistration":false,"contentFormat":["markdown","plaintext"]},"synchronization":{"dynamicRegistration":false,"didSave":true,"willSaveWaitUntil":true,"willSave":true},"publishDiagnostics":{"relatedInformation":true,"tagSupport":{"valueSet":[1,2]}},"codeAction":{"isPreferredSupport":true,"dynamicRegistration":false,"resolveSupport":{"properties":["edit"]},"dataSupport":true,"codeActionLiteralSupport":{"codeActionKind":{"valueSet":["","quickfix","refactor","refactor.extract","refactor.inline","refactor.rewrite","source","source.organizeImports"]}}},"references":{"dynamicRegistration":false},"implementation":{"linkSupport":true},"declaration":{"linkSupport":true},"definition":{"linkSupport":true},"semanticTokens":{"overlappingTokenSupport":true,"multilineTokenSupport":false,"serverCancelSupport":false,"augmentsSyntaxTokens":true,"tokenModifiers":["declaration","definition","readonly","static","deprecated","abstract","async","modification","documentation","defaultLibrary"],"requests":{"range":false,"full":{"delta":true}},"dynamicRegistration":false,"tokenTypes":["namespace","type","class","enum","interface","struct","typeParameter","parameter","variable","property","enumMember","event","function","method","macro","keyword","modifier","comment","string","number","regexp","operator","decorator"],"formats":["relative"]},"rename":{"dynamicRegistration":false,"prepareSupport":true},"completion":{"dynamicRegistration":false,"completionItemKind":{"valueSet":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25]},"contextSupport":false,"completionItem":{"snippetSupport":false,"commitCharactersSupport":false,"preselectSupport":false,"deprecatedSupport":false,"documentationFormat":["markdown","plaintext"]}},"signatureHelp":{"dynamicRegistration":false,"signatureInformation":{"documentationFormat":["markdown","plaintext"],"activeParameterSupport":true,"parameterInformation":{"labelOffsetSupport":true}}},"typeDefinition":{"linkSupport":true}},"window":{"workDoneProgress":true,"showMessage":{"messageActionItem":{"additionalPropertiesSupport":false}},"showDocument":{"support":true}},"workspace":{"symbol":{"dynamicRegistration":false,"hierarchicalWorkspaceSymbolSupport":true,"symbolKind":{"valueSet":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26]}},"configuration":true,"applyEdit":true,"workspaceEdit":{"resourceOperations":["rename","create","delete"]},"semanticTokens":{"refreshSupport":true},"workspaceFolders":true,"didChangeWatchedFiles":{"dynamicRegistration":false,"relativePatternSupport":true}}},"workspaceFolders":null,"rootPath":null,"processId":4740,"clientInfo":{"version":"0.9.1","name":"Neovim"},"initializationOptions":{}},"id":1,"jsonrpc":"2.0","method":"initialize"}"#).build();

    let stdin = Builder::new().read(b"Hello world\r\n").build();
    let stdout = Builder::new().write(b"fail\r\n").build();
    let stderr = Builder::new()
        .write(b"gopls: invalid header line \"Hello world\"\n")
        .write(b"failure")
        .build();

    let mut child = entrypoint::run(
        "quay.io/pvlerick/gopls:0.14.2-r0".to_string(),
        &main_dir_path,
        stdin,
        stdout,
        stderr,
        CancellationToken::new(),
    )
    .await
    .unwrap();

    let _ = child.wait().await;

    std::thread::sleep(std::time::Duration::from_secs(5));
    fs::remove_dir_all(main_dir_path).await?;

    Ok(())
}
