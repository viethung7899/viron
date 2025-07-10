use std::path::PathBuf;

use anyhow::{Ok, Result};
use async_trait::async_trait;
use lsp_types::{
    GotoDefinitionResponse, InitializeResult, InitializedParams, Location,
    PublishDiagnosticsParams,
    notification::{Initialized, Notification, PublishDiagnostics},
    request::{Initialize, Request},
};
use serde_json::Value;

use crate::{
    input::actions::{CompositeExecutable, GoToPosition, OpenBuffer, UpdateDiagnostics},
    service::lsp::{
        LspAction,
        client::{LspClient, LspClientState},
        messages::InboundNotification,
    },
};

#[async_trait]
pub trait LspMessageHandler: Send + Sync {
    async fn handle_client(&self, _client: &mut LspClient) -> Result<()> {
        return Ok(());
    }

    fn get_lsp_action(&self) -> Option<LspAction> {
        return None;
    }
}

#[derive(Debug)]
struct UnknownResponse {
    method: String,
    result: Value,
}

#[async_trait]
impl LspMessageHandler for UnknownResponse {}

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

        action.add(OpenBuffer::new(PathBuf::from(location.uri.as_str())));

        let position = location.range.start;
        action.add(GoToPosition::new(
            position.line as usize,
            position.character as usize,
        ));

        Some(Box::new(action))
    }
}

pub fn parse_response(method: &str, result: Value) -> Result<Box<dyn LspMessageHandler>> {
    let handler: Box<dyn LspMessageHandler> = match method {
        Initialize::METHOD => Box::new(serde_json::from_value::<InitializeResult>(result)?),
        _ => Box::new(UnknownResponse {
            method: method.to_string(),
            result,
        }),
    };
    Ok(handler)
}

#[async_trait]
impl LspMessageHandler for InboundNotification {}

#[async_trait]
impl LspMessageHandler for PublishDiagnosticsParams {
    fn get_lsp_action(&self) -> Option<LspAction> {
        Some(Box::new(UpdateDiagnostics::new(
            self.uri.as_str().to_string(),
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
