mod client;
mod types;

pub use types::Diagnostic;

use anyhow::Result;

use crate::{
    core::message::Message,
    input::actions,
    service::lsp::client::{InboundMessage, LspClient, NotificationKind},
};

#[derive(Debug, Default)]
pub struct LspService {
    client: Option<LspClient>,
    command: Option<String>,
    diagnostics: Vec<Diagnostic>,
    enabled: bool,
}

type LspAction = Box<dyn actions::Executable>;

impl LspService {
    pub fn new() -> Self {
        Self {
            client: None,
            command: None,
            diagnostics: Vec::new(),
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

    pub async fn start_server(
        &mut self,
        server_cmd: &str,
        file_path: &str,
        contents: &str,
    ) -> Result<()> {
        if self.client.is_some() || !self.enabled {
            return Ok(());
        }

        let mut client = LspClient::start(server_cmd, &[]).await?;

        client.initialize().await?;
        log::info!("LSP client initialized");
        
        let uri = format!("file://{}", file_path);

        client.did_open(&uri, contents).await?;
        log::info!("LSP client did open file: {}", file_path);

        self.client = Some(client);
        self.command = Some(server_cmd.to_string());
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(ref mut client) = self.client {
            client.shutdown().await?;
        }
        self.client = None;
        self.command = None;
        Ok(())
    }

    pub async fn restart(
        &mut self,
        server_cmd: &str,
        file_path: &str,
        contents: &str,
    ) -> Result<()> {
        // Shutdown existing client
        self.shutdown().await?;

        // Enable and start new client
        self.enabled = true;
        self.start_server(server_cmd, file_path, contents).await
    }

    pub async fn handle_message(&mut self) -> Option<LspAction> {
        let client = self.client.as_mut()?;

        let Ok(Some((messages, _))) = client.receive_response().await else {
            return None;
        };
        log::info!("Messages: {:#?}", messages);
        self.process_messages(messages).await
    }

    pub async fn process_messages(&mut self, message: InboundMessage) -> Option<LspAction> {
        match message {
            InboundMessage::Notification(notification) => {
                return self.handle_notification(notification).await;
            }
            InboundMessage::Error(err) => {
                let message = Message::error(format!("LSP Error {}", err.message));
                return Some(Box::new(actions::ShowMessage(message)));
            }
            InboundMessage::ProcessingError(err) => {
                log::error!("LSP processing error {}", err);
            }
            _ => {}
        }
        None
    }

    async fn handle_notification(&mut self, notification: NotificationKind) -> Option<LspAction> {
        match notification {
            NotificationKind::ShowMessage(msg) => {
                let message = match msg.typ {
                    types::MessageType::Info | types::MessageType::Log => {
                        Message::info(msg.message)
                    }
                    _ => Message::error(msg.message),
                };
                Some(Box::new(actions::ShowMessage(message)))
            }
            NotificationKind::LogMessage(msg) => {
                match msg.typ {
                    types::MessageType::Error => log::error!("{}", msg.message),
                    types::MessageType::Info | types::MessageType::Log => {
                        log::info!("{}", msg.message)
                    }
                    types::MessageType::Warning => log::warn!("{}", msg.message),
                };
                None
            }
            NotificationKind::PublishDiagnostics(diagnostics) => {
                self.diagnostics = diagnostics.diagnostics;
                if self.diagnostics.is_empty() {
                    return None;
                }
                let message =
                    Message::info(format!("Found {} diagnotics in", self.diagnostics.len()));
                Some(Box::new(actions::ShowMessage(message)))
            }
        }
    }

    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
