mod client;
mod message_handler;
mod messages;
mod params;
mod util;
mod version;

use std::collections::HashMap;

use crate::core::language::Language;
use crate::service::lsp::client::LspClientState;
use anyhow::Result;
use lsp_types::Diagnostic;
use crate::actions::core::Executable;

pub(crate) use crate::service::lsp::client::LspClient;

#[derive(Debug, Default)]
pub struct LspService {
    client: Option<LspClient>,
    diagnostics: HashMap<String, Vec<Diagnostic>>,
    enabled: bool,
}

type LspAction = Box<dyn Executable>;

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

        let Ok(mut client) = LspClient::new(language, &[]).await else {
            self.shutdown().await?;
            return Ok(None);
        };

        client.initialize().await?;

        while client.state != LspClientState::Initialized {
            let _ = client.get_lsp_action().await?;
        }

        self.client = Some(client);
        Ok(self.client.as_mut())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        let client = std::mem::take(&mut self.client);
        if let Some(client) = client {
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

    pub fn get_diagnostics(&self, uri: &str) -> &[Diagnostic] {
        self.diagnostics
            .get(uri)
            .map(|d| d.as_slice())
            .unwrap_or_default()
    }

    pub fn update_diagnostics(&mut self, path: &str, diagnostics: Vec<Diagnostic>) {
        self.diagnostics.insert(path.to_string(), diagnostics);
    }
}
