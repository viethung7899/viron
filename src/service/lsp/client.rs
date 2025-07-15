use crate::core::document::Document;
use crate::core::language::Language;
use crate::service::lsp::message_handler::{parse_notification, parse_response};
use crate::service::lsp::messages::{lsp_receive, lsp_send, InboundMessage, OutboundMessage};
use crate::service::lsp::params::get_initialize_params;
use crate::service::lsp::LspAction;
use anyhow::{Context, Result};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument, Exit,
    Notification,
};
use lsp_types::request::{
    DocumentDiagnosticRequest, GotoDefinition, Initialize, Request, Shutdown,
};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, DocumentDiagnosticParams, GotoDefinitionParams, Position, Range,
    ServerCapabilities, TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
    VersionedTextDocumentIdentifier,
};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::AtomicI32;
use std::sync::Arc;
use std::{
    process::Stdio,
    sync::atomic::{self},
};

use crate::service::lsp::util::calculate_changes;
use crate::service::lsp::version::VersionedContents;
use tokio::process::Child;
use tokio::sync::Mutex;
use tokio::{
    io::{BufReader, BufWriter},
    process::Command,
    sync::mpsc,
};
use tree_sitter::InputEdit;

static ID: AtomicI32 = AtomicI32::new(1);
const CHANNEL_SIZE: usize = 32;

pub fn next_id() -> i32 {
    ID.fetch_add(1, atomic::Ordering::SeqCst)
}

#[derive(Debug, Clone, PartialEq)]
pub enum LspClientState {
    Uninitialized,
    Initializing,
    Initialized,
}

#[derive(Debug)]
pub struct LspClient {
    pub(super) language: Language,
    pub(super) state: LspClientState,
    pub(super) server_capabilities: Option<ServerCapabilities>,

    request_sender: mpsc::Sender<OutboundMessage>,
    response_receiver: mpsc::Receiver<InboundMessage>,
    pending_responses: HashMap<i32, String>,

    process: Arc<Mutex<Option<Child>>>,

    versioned_contents: VersionedContents,
}

impl LspClient {
    pub async fn new(language: Language, args: &[&str]) -> Result<Self> {
        let command = language
            .get_language_server()
            .context("Language is not supported")?;
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;

        let (request_sender, mut request_receiver) = mpsc::channel::<OutboundMessage>(CHANNEL_SIZE);
        let (response_sender, response_receiver) = mpsc::channel::<InboundMessage>(CHANNEL_SIZE);

        // Send requests from editor into LSP's stdin
        tokio::spawn(async move {
            let mut writer = BufWriter::new(stdin);
            while let Some(message) = request_receiver.recv().await {
                lsp_send(&mut writer, message).await?;
            }
            anyhow::Ok(())
        });

        // Sends responses from LSP's stdout to the editor
        let sender = response_sender.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);

            while let Ok(message) = lsp_receive(&mut reader).await {
                let Some(message) = message else {
                    continue;
                };
                sender.send(message).await?;
            }

            anyhow::Ok(())
        });

        Ok(LspClient {
            language,
            state: LspClientState::Uninitialized,
            request_sender,
            response_receiver,
            server_capabilities: None,
            pending_responses: HashMap::new(),
            process: Arc::new(Mutex::new(Some(child))),
            versioned_contents: VersionedContents::default(),
        })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.state = LspClientState::Initializing;
        self.send_request::<Initialize>(get_initialize_params(), true)
            .await?;
        Ok(())
    }

    pub async fn did_open(&mut self, document: &Document) -> Result<()> {
        let Some(uri) = document.get_uri() else {
            return Ok(());
        };

        self.versioned_contents
            .update_document(&uri, document.buffer.to_string());

        self.send_notification::<DidOpenTextDocument>(
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: Uri::from_str(&uri)?,
                    version: document.version as i32,
                    text: document.buffer.to_string(),
                    language_id: document.language.to_str().to_string(),
                },
            },
            false,
        )
        .await?;

        Ok(())
    }

    pub async fn did_save(&mut self, document: &Document) -> Result<()> {
        let Some(path) = document.full_path_string() else {
            return Ok(());
        };

        self.send_notification::<DidSaveTextDocument>(
            DidSaveTextDocumentParams {
                text: Some(document.buffer.to_string()),
                text_document: TextDocumentIdentifier {
                    uri: Uri::from_str(&path)?,
                },
            },
            false,
        )
        .await?;
        Ok(())
    }

    pub async fn did_close(&mut self, document: &Document) -> Result<()> {
        let Some(uri) = document.get_uri() else {
            return Ok(());
        };

        self.send_notification::<DidCloseTextDocument>(
            DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier {
                    uri: Uri::from_str(&uri)?,
                },
            },
            false,
        )
        .await?;

        Ok(())
    }

    pub async fn did_change(&mut self, document: &Document) -> Result<()> {
        let Some(uri) = document.get_uri() else {
            return Ok(());
        };
        self.request_diagnostics(document).await?;

        let content = document.buffer.to_string();

        let sync_kind = self
            .server_capabilities
            .as_ref()
            .and_then(|capabilities| capabilities.text_document_sync.as_ref())
            .and_then(|sync| match sync {
                TextDocumentSyncCapability::Kind(kind) => Some(kind),
                TextDocumentSyncCapability::Options(option) => option.change.as_ref(),
            })
            .cloned()
            .unwrap_or(TextDocumentSyncKind::FULL);

        let content_changes = match sync_kind {
            TextDocumentSyncKind::FULL => {
                vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: document.buffer.to_string(),
                }]
            }
            TextDocumentSyncKind::INCREMENTAL => {
                let old_content = self.versioned_contents.get_content(&uri);
                calculate_changes(old_content, &content)
            }
            _ => {
                return Ok(());
            }
        };

        self.versioned_contents.update_document(&uri, content);
        let version = self.versioned_contents.get_version(&uri);

        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: Uri::from_str(&uri)?,
                version,
            },
            content_changes,
        };

        self.send_notification::<DidChangeTextDocument>(params, false)
            .await?;

        Ok(())
    }

    pub async fn goto_definition(
        &mut self,
        document: &Document,
        line: usize,
        character: usize,
    ) -> Result<()> {
        let Some(uri) = document.get_uri() else {
            return Ok(());
        };
        self.send_request::<GotoDefinition>(
            GotoDefinitionParams {
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier {
                        uri: Uri::from_str(&uri)?,
                    },
                    position: Position {
                        line: line as u32,
                        character: character as u32,
                    },
                },
            },
            false,
        )
        .await?;

        Ok(())
    }

    pub async fn request_diagnostics(&mut self, document: &Document) -> Result<Option<i32>> {
        let Some(uri) = document.get_uri() else {
            return Ok(None);
        };

        let can_request_diagnostics = self
            .server_capabilities
            .as_ref()
            .map(|capabilities| capabilities.diagnostic_provider.is_some())
            .unwrap_or(false);

        if !can_request_diagnostics {
            return Ok(None);
        }

        let params = DocumentDiagnosticParams {
            text_document: TextDocumentIdentifier {
                uri: Uri::from_str(&uri)?,
            },
            identifier: None,
            previous_result_id: None,
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        let id = self
            .send_request::<DocumentDiagnosticRequest>(params, false)
            .await?;
        Ok(Some(id))
    }

    async fn send_request<R: Request>(&mut self, params: R::Params, force: bool) -> Result<i32> {
        if self.state != LspClientState::Initialized && !force {
            return Err(anyhow::anyhow!("LSP client is not initialized"));
        }
        let id = next_id();
        let method = R::METHOD.to_string();
        let params = serde_json::to_value(params)?;

        log::info!("Sending {} request with {:?}", method, params);

        self.pending_responses.insert(id, method.to_string());
        self.request_sender
            .send(OutboundMessage {
                id: Some(id),
                method: method.to_string(),
                params,
            })
            .await?;

        log::info!("Request {} sent", method);
        Ok(id)
    }

    pub async fn send_notification<N: Notification>(
        &mut self,
        params: N::Params,
        force: bool,
    ) -> Result<()> {
        if self.state != LspClientState::Initialized && !force {
            return Err(anyhow::anyhow!("LSP client is not initialized"));
        }
        let method = N::METHOD.to_string();
        let params = serde_json::to_value(params)?;

        log::info!("Sending {} notification with {:?}", method, params);

        self.request_sender
            .send(OutboundMessage {
                id: None,
                method: method.to_string(),
                params,
            })
            .await?;

        log::info!("Notification {} sent", method);

        Ok(())
    }

    pub async fn get_lsp_action(&mut self) -> Result<Option<LspAction>> {
        let Ok(message) = self.response_receiver.try_recv() else {
            return Ok(None);
        };

        let handler = match message {
            InboundMessage::Response(response) => {
                let Some(method) = self.pending_responses.get(&response.id) else {
                    return Ok(None);
                };
                let Some(result) = response.result.to_owned() else {
                    return Ok(None);
                };
                parse_response(method, result)?
            }
            InboundMessage::Notification(notification) => parse_notification(notification)?,
        };

        handler.handle_client(self).await?;
        Ok(handler.get_lsp_action())
    }

    pub async fn is_running(&self) -> bool {
        if let Ok(mut process) = self.process.try_lock() {
            if let Some(child) = process.as_mut() {
                match child.try_wait() {
                    Ok(Some(_)) => false, // Process has exited
                    Ok(None) => true,     // Process is still running
                    Err(_) => false,      // Error checking process
                }
            } else {
                false
            }
        } else {
            true // Assume running if we can't check
        }
    }

    pub async fn kill(&mut self) -> Result<()> {
        let mut process = self.process.lock().await;
        if let Some(mut child) = process.take() {
            child.kill().await?;
        }
        Ok(())
    }

    pub async fn shutdown(mut self) -> Result<()> {
        // Send shutdown request and wait for response
        let shutdown_id = self.send_request::<Shutdown>((), true).await?;

        // Wait for shutdown response (with timeout)
        let timeout_duration = std::time::Duration::from_secs(5);
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout_duration {
            if let Some(InboundMessage::Response(response)) = self.response_receiver.recv().await {
                if response.id == shutdown_id {
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // Send exit notification
        self.send_notification::<Exit>((), true).await?;

        // Give the process a moment to exit gracefully
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Force kill if still running
        if self.is_running().await {
            self.kill().await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use lsp_types::Uri;
    use std::str::FromStr;

    #[test]
    fn test_uri() {
        let uri = Uri::from_str("file:///tmp/sample").unwrap();
        assert!(uri.is_absolute());
        assert_eq!(uri.to_string(), "file:///tmp/sample");
    }
}
