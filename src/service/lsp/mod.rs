mod client;
mod params;
pub mod types;

use std::collections::HashMap;
use std::path::PathBuf;
use types::Diagnostic;

use crate::core::language::Language;
use crate::service::lsp::types::DefinitionResult;
use crate::{
    input::actions,
    service::lsp::client::{InboundMessage, LspClient, NotificationKind},
};
use anyhow::Result;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct LspService {
    client: Option<LspClient>,
    diagnostics: HashMap<String, Vec<Diagnostic>>,
    enabled: bool,
}

type LspAction = Box<dyn actions::Executable>;

impl LspService {
    pub fn new() -> Self {
        Self {
            client: None,
            diagnostics: HashMap::new(),
            enabled: true,
        }
    }

    pub fn get_client_mut(&mut self) -> Option<&mut LspClient> {
        if !self.enabled {
            return None;
        }
        self.client.as_mut()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_running(&self) -> bool {
        self.client.is_some()
    }

    pub async fn start_server(&mut self, language: Language) -> Result<Option<&mut LspClient>> {
        if !self.enabled {
            return Ok(None);
        }

        // Check if the language server is already running for the given language
        if let Some(old_client) = &self.client {
            if old_client.language == language {
                return Ok(self.client.as_mut());
            } else {
                self.shutdown().await?;
            }
        }

        let Ok(mut client) = LspClient::start(language, &[]).await else {
            self.shutdown().await?;
            return Ok(None);
        };

        client.initialize().await?;
        log::info!("LSP client initialized");

        self.client = Some(client);
        Ok(self.client.as_mut())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(ref mut client) = self.client {
            client.shutdown().await?;
        }
        self.client = None;
        Ok(())
    }

    pub async fn restart(&mut self, language: Language) -> Result<Option<&mut LspClient>> {
        // Shutdown existing client
        self.shutdown().await?;

        // Enable and start new client
        self.enabled = true;
        self.start_server(language).await
    }

    pub async fn handle_message(&mut self) -> Option<LspAction> {
        let client = self.client.as_mut()?;

        let Ok(Some((messages, method))) = client.receive_response().await else {
            return None;
        };
        log::info!("Messages: {:#?}", messages);
        self.process_messages(messages, method).await
    }

    pub async fn process_messages(
        &mut self,
        message: InboundMessage,
        method: Option<String>,
    ) -> Option<LspAction> {
        match message {
            InboundMessage::Notification(notification) => {
                return self.handle_notification(notification).await;
            }
            InboundMessage::Error(err) => {
                log::error!("LSP Error: {}", err.message);
                return None;
            }
            InboundMessage::ProcessingError(err) => {
                log::error!("LSP processing error {}", err);
            }
            InboundMessage::Response(response) => {
                let Some(method) = &method else {
                    return None;
                };
                match self.handle_response(method, response.result) {
                    Ok(action) => {
                        return action;
                    }
                    Err(_err) => {
                        log::warn!("Unhandled LSP response for method: {}", method);
                    }
                }
            }
            _ => {}
        }
        None
    }

    async fn handle_notification(&mut self, notification: NotificationKind) -> Option<LspAction> {
        match notification {
            NotificationKind::ShowMessage(msg) => {
                match msg.type_ {
                    types::MessageType::Info => log::info!("{}", msg.message),
                    types::MessageType::Warning => log::warn!("{}", msg.message),
                    types::MessageType::Error => log::error!("{}", msg.message),
                    types::MessageType::Log => log::debug!("{}", msg.message),
                };
                None
            }
            NotificationKind::LogMessage(msg) => {
                match msg.type_ {
                    types::MessageType::Error => log::error!("{}", msg.message),
                    types::MessageType::Info | types::MessageType::Log => {
                        log::info!("{}", msg.message)
                    }
                    types::MessageType::Warning => log::warn!("{}", msg.message),
                };
                None
            }
            NotificationKind::PublishDiagnostics(diagnostics) => {
                self.diagnostics
                    .insert(diagnostics.uri.unwrap_or_default(), diagnostics.diagnostics);
                Some(Box::new(actions::RefreshBuffer))
            }
        }
    }

    pub fn handle_response(&self, method: &str, result: Value) -> Result<Option<LspAction>> {
        match method {
            "textDocument/definition" => {
                let result = serde_json::from_value::<Option<DefinitionResult>>(result)?;
                let Some(result) = result else {
                    return Ok(None);
                };
                if let Some(location) = result.get_locations().first() {
                    let mut action = actions::CompositeExecutable::new();
                    let file_path = location.uri.strip_prefix("file://").unwrap();
                    action.add(actions::OpenBuffer::new(PathBuf::from(file_path)));
                    let position = &location.range.start;
                    action.add(actions::GoToPosition::new(
                        position.line,
                        position.character,
                    ));
                    return Ok(Some(Box::new(action)));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    pub fn get_diagnostics(&self, uri: &str) -> &[Diagnostic] {
        self.diagnostics
            .get(uri)
            .map(|d| d.as_slice())
            .unwrap_or_default()
    }
}
