use std::path::PathBuf;

use anyhow::{Ok, Result};
use async_trait::async_trait;
use lsp_types::request::DocumentDiagnosticRequest;
use lsp_types::{
    notification::{Initialized, Notification, PublishDiagnostics}, request::{Initialize, Request}, DocumentDiagnosticReport, GotoDefinitionResponse,
    InitializeResult, InitializedParams,
    Location,
    PublishDiagnosticsParams,
};
use serde_json::Value;

use crate::{
    service::lsp::{
        client::{LspClient, LspClientState},
        messages::InboundNotification,
        LspAction,
    },
};
use crate::actions::{buffer, lsp, movement};
use crate::actions::core::CompositeExecutable;

#[async_trait]
pub trait LspMessageHandler: Send + Sync {
    async fn handle_client(&self, _client: &mut LspClient) -> Result<()> {
        Ok(())
    }

    fn get_lsp_action(&self) -> Option<LspAction> {
        None
    }
}

#[derive(Debug)]
struct UnknownResponse {
    method: String,
    result: Value,
}

#[async_trait]
impl LspMessageHandler for UnknownResponse {
    fn get_lsp_action(&self) -> Option<LspAction> {
        log::info!(
            "Unknown LSP response: \nMethod:{}\nResult:{:?}",
            self.method,
            self.result
        );
        None
    }
}

#[async_trait]
impl LspMessageHandler for InitializeResult {
    async fn handle_client(&self, client: &mut LspClient) -> Result<()> {
        client.server_capabilities = Some(self.capabilities.clone());
        client
            .send_notification::<Initialized>(InitializedParams {}, true)
            .await?;
        client.state = LspClientState::Initialized;
        Ok(())
    }
}

impl LspMessageHandler for GotoDefinitionResponse {
    fn get_lsp_action(&self) -> Option<LspAction> {
        let location = match self {
            GotoDefinitionResponse::Scalar(location) => location.clone(),
            GotoDefinitionResponse::Array(locations) => locations.first().cloned()?,
            GotoDefinitionResponse::Link(location_links) => {
                location_links.first().map(|link| Location {
                    uri: link.target_uri.clone(),
                    range: link.target_range,
                })?
            }
        };

        let mut action = CompositeExecutable::new();

        action.add(buffer::OpenBuffer::new(PathBuf::from(location.uri.as_str())));

        let position = location.range.start;
        action.add(movement::GoToPosition::new(
            position.line as usize,
            position.character as usize,
        ));

        Some(Box::new(action))
    }
}

impl LspMessageHandler for DocumentDiagnosticReport {
    fn get_lsp_action(&self) -> Option<LspAction> {
        match self {
            DocumentDiagnosticReport::Full(full) => Some(Box::new(lsp::UpdateDiagnostics::new(
                None,
                full.full_document_diagnostic_report.items.clone(),
            ))),
            _ => None,
        }
    }
}

pub fn parse_response(method: &str, result: Value) -> Result<Box<dyn LspMessageHandler>> {
    let handler: Box<dyn LspMessageHandler> = match method {
        Initialize::METHOD => Box::new(serde_json::from_value::<InitializeResult>(result)?),
        DocumentDiagnosticRequest::METHOD => {
            Box::new(serde_json::from_value::<DocumentDiagnosticReport>(result)?)
        }
        _ => Box::new(UnknownResponse {
            method: method.to_string(),
            result,
        }),
    };
    Ok(handler)
}

impl LspMessageHandler for InboundNotification {
    fn get_lsp_action(&self) -> Option<LspAction> {
        log::info!(
            "Unhandled LSP notification: \nMethod: {}\nResult: {:?}",
            self.method,
            self.params
        );
        None
    }
}

impl LspMessageHandler for PublishDiagnosticsParams {
    fn get_lsp_action(&self) -> Option<LspAction> {
        Some(Box::new(lsp::UpdateDiagnostics::new(
            Some(self.uri.to_string()),
            self.diagnostics.clone(),
        )))
    }
}

pub fn parse_notification(notification: InboundNotification) -> Result<Box<dyn LspMessageHandler>> {
    let Some(params) = &notification.params else {
        return Ok(Box::new(notification));
    };

    let params = params.to_owned();

    let handler: Box<dyn LspMessageHandler> = match notification.method.as_str() {
        PublishDiagnostics::METHOD => {
            Box::new(serde_json::from_value::<PublishDiagnosticsParams>(params)?)
        }
        _ => Box::new(notification),
    };
    Ok(handler)
}
