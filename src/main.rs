mod cursor;
mod formatter;
mod syntax;

use std::{collections::HashMap, sync::Mutex};

use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::{Error, Result},
    ls_types::*,
};

use crate::{formatter::get_completions, syntax::parse};

struct Backend {
    client: Client,
    open_files: Mutex<HashMap<Uri, String>>,
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["custom.notification".to_string()],
                    ..Default::default()
                }),
                completion_provider: Some(CompletionOptions {
                    ..Default::default()
                }),
                document_formatting_provider: Some(OneOf::Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.open_files
            .lock()
            .unwrap()
            .insert(params.text_document.uri, params.text_document.text);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.open_files
            .lock()
            .unwrap()
            .remove(&params.text_document.uri);
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.open_files.lock().unwrap().insert(
            params.text_document.uri,
            params.content_changes.remove(0).text,
        );
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<LSPAny>> {
        if params.command == "custom.notification" {
            self.client
                .log_message(MessageType::INFO, "custom.notification executed")
                .await;
            self.client
                .show_message(MessageType::INFO, "Hello world!")
                .await;
            Ok(None)
        } else {
            Err(Error::invalid_request())
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        fn range_contains(range: Range, position: Position) -> bool {
            position >= range.start && position < range.end
        }

        let content = self.read_file(params.text_document_position.text_document.uri)?;
        let syntax = parse(&content);
        // The actual cursor position can often be the first character after the current node
        let position = Position::new(
            params.text_document_position.position.line,
            params
                .text_document_position
                .position
                .character
                .saturating_sub(1),
        );
        let Some((_, node)) = syntax
            .iter()
            .find(|(range, _)| range_contains(*range, position))
        else {
            return Ok(None);
        };
        Ok(Some(CompletionResponse::Array(get_completions(&node.kind))))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let contents = self.read_file(uri)?;
        let edits = crate::formatter::format(&contents);
        Ok(Some(edits))
    }
}

impl Backend {
    fn read_file(&self, uri: Uri) -> Result<String> {
        if let Some(entry) = self.open_files.lock().unwrap().get(&uri) {
            return Ok(entry.clone());
        }

        let Some(path) = uri.to_file_path() else {
            return Err(Error::invalid_params("Cannot read uri {uri:?}"));
        };
        std::fs::read_to_string(path).map_err(|_| Error::internal_error())
    }
}

fn main() {
    let stdin = blocking::Unblock::new(std::io::stdin());
    let stdout = blocking::Unblock::new(std::io::stdout());

    let (service, socket) = LspService::new(|client| Backend {
        client,
        open_files: Default::default(),
    });
    futures_lite::future::block_on(Server::new(stdin, stdout, socket).serve(service));
}
