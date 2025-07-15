use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Serialize, Deserialize)]
pub struct OutboundMessage {
    pub(crate) id: Option<i32>,
    pub(crate) method: String,
    pub(crate) params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundResponse {
    pub id: i32,
    pub result: Option<Value>,
    pub error: Option<ResponseError>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundNotification {
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundError {
    pub id: i32,
    pub error: ResponseError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InboundMessage {
    Response(InboundResponse),
    Notification(InboundNotification),
}

pub async fn lsp_send<W: Unpin + AsyncWrite>(
    writer: &mut W,
    message: OutboundMessage,
) -> anyhow::Result<()> {
    let mut body = json!({
        "jsonrpc": "2.0",
        "method": message.method,
        "params": message.params,
    });
    if let Some(id) = message.id {
        body["id"] = json!(id);
    }
    let body = serde_json::to_string(&body)?;
    let content = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
    log::info!("=> {}", body);
    writer.write_all(content.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn lsp_receive<R: Unpin + AsyncBufRead>(
    reader: &mut R,
) -> anyhow::Result<Option<InboundMessage>> {
    let mut line = String::new();
    let read_size = reader.read_line(&mut line).await?;
    if read_size <= 0 {
        return Ok(None);
    }
    let length = line
        .strip_prefix("Content-Length: ")
        .context("Expected Content-Length header")?
        .trim()
        .parse::<usize>()?;
    reader.read_line(&mut line).await?;

    let mut body = vec![0; length];
    reader.read_exact(&mut body).await?;

    log::info!("<= {}", String::from_utf8_lossy(&body));

    let message: InboundMessage = serde_json::from_slice(&body)?;
    Ok(Some(message))
}
