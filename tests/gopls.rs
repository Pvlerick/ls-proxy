use std::error::Error;

use tokio::{fs::File, io::AsyncWriteExt};

use crate::common::{create_tmp_dir, spawn_app};

mod common;

#[tokio::test]
async fn gopls() -> Result<(), Box<dyn Error + Send + Sync>> {
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

    let mut sut = spawn_app(
        "quay.io/pvlerick/gopls:0.14.2-r0".to_string(),
        main_dir_path,
    )
    .unwrap();

    sut.write_stdin(r#"Content-Length: 3111\u{000D}\u{000A}{"params":{"trace":"off","rootUri":null,"capabilities":{"textDocument":{"documentHighlight":{"dynamicRegistration":false},"documentSymbol":{"dynamicRegistration":false,"symbolKind":{"valueSet":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26]},"hierarchicalDocumentSymbolSupport":true},"callHierarchy":{"dynamicRegistration":false},"hover":{"dynamicRegistration":false,"contentFormat":["markdown","plaintext"]},"synchronization":{"dynamicRegistration":false,"didSave":true,"willSaveWaitUntil":true,"willSave":true},"publishDiagnostics":{"relatedInformation":true,"tagSupport":{"valueSet":[1,2]}},"codeAction":{"isPreferredSupport":true,"dynamicRegistration":false,"resolveSupport":{"properties":["edit"]},"dataSupport":true,"codeActionLiteralSupport":{"codeActionKind":{"valueSet":["","quickfix","refactor","refactor.extract","refactor.inline","refactor.rewrite","source","source.organizeImports"]}}},"references":{"dynamicRegistration":false},"implementation":{"linkSupport":true},"declaration":{"linkSupport":true},"definition":{"linkSupport":true},"semanticTokens":{"overlappingTokenSupport":true,"multilineTokenSupport":false,"serverCancelSupport":false,"augmentsSyntaxTokens":true,"tokenModifiers":["declaration","definition","readonly","static","deprecated","abstract","async","modification","documentation","defaultLibrary"],"requests":{"range":false,"full":{"delta":true}},"dynamicRegistration":false,"tokenTypes":["namespace","type","class","enum","interface","struct","typeParameter","parameter","variable","property","enumMember","event","function","method","macro","keyword","modifier","comment","string","number","regexp","operator","decorator"],"formats":["relative"]},"rename":{"dynamicRegistration":false,"prepareSupport":true},"completion":{"dynamicRegistration":false,"completionItemKind":{"valueSet":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25]},"contextSupport":false,"completionItem":{"snippetSupport":false,"commitCharactersSupport":false,"preselectSupport":false,"deprecatedSupport":false,"documentationFormat":["markdown","plaintext"]}},"signatureHelp":{"dynamicRegistration":false,"signatureInformation":{"documentationFormat":["markdown","plaintext"],"activeParameterSupport":true,"parameterInformation":{"labelOffsetSupport":true}}},"typeDefinition":{"linkSupport":true}},"window":{"workDoneProgress":true,"showMessage":{"messageActionItem":{"additionalPropertiesSupport":false}},"showDocument":{"support":true}},"workspace":{"symbol":{"dynamicRegistration":false,"hierarchicalWorkspaceSymbolSupport":true,"symbolKind":{"valueSet":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26]}},"configuration":true,"applyEdit":true,"workspaceEdit":{"resourceOperations":["rename","create","delete"]},"semanticTokens":{"refreshSupport":true},"workspaceFolders":true,"didChangeWatchedFiles":{"dynamicRegistration":false,"relativePatternSupport":true}}},"workspaceFolders":null,"rootPath":null,"processId":4740,"clientInfo":{"version":"0.9.1","name":"Neovim"},"initializationOptions":{}},"id":1,"jsonrpc":"2.0","method":"initialize"}"#);

    assert_eq!("", main_file_path.to_str().unwrap());

    Ok(())
}
