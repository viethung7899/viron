use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process;
use std::{
    process::Stdio,
    sync::atomic::{self, AtomicUsize},
};
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{ChildStdin, ChildStdout, Command},
    sync::mpsc,
};

use crate::log;

mod types;

static ID: AtomicUsize = AtomicUsize::new(1);

pub fn next_id() -> usize {
    ID.fetch_add(1, atomic::Ordering::SeqCst)
}

#[derive(Debug)]
pub struct NotificationRequest {
    method: String,
    params: Value,
}

#[derive(Debug)]
pub struct Request {
    id: i64,
    method: String,
    params: Value,
}

impl Request {
    pub fn new(method: &str, params: Value) -> Self {
        Self {
            id: next_id() as i64,
            method: method.to_string(),
            params,
        }
    }
}

#[derive(Debug)]
pub enum OutboundMessage {
    Request(Request),
    Notification(NotificationRequest),
}

#[derive(Debug, Clone)]
pub struct ResponseMessage {
    pub id: i64,
    pub result: Value,
}

#[derive(Debug)]
pub struct Notification {
    method: String,
    params: Value,
}

#[derive(Debug)]
pub struct ResponseError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug)]
pub enum InboundMessage {
    Message(ResponseMessage),
    Notification(Notification),
    Error(ResponseError),
    ProcessingError(String),
}

pub async fn start_lsp() -> Result<LspClient> {
    let mut child = Command::new("rust-analyzer")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take().context("Failed to get stdin")?;
    let stdout = child.stdout.take().context("Failed to get stdout")?;
    let stderr = child.stderr.take().context("Failed to get stderr")?;

    let (request_tx, mut request_rx) = mpsc::channel::<OutboundMessage>(32);
    let (response_tx, response_rx) = mpsc::channel::<InboundMessage>(32);

    // Send requests from editor into LSP's stdin
    let rtx = response_tx.clone();
    tokio::spawn(async move {
        let mut writer = BufWriter::new(stdin);
        while let Some(message) = request_rx.recv().await {
            log!("[lsp] editor requested to send message: {:?}", message);
            match message {
                OutboundMessage::Request(request) => {
                    if let Err(error) = lsp_send_request(&mut writer, &request).await {
                        rtx.send(InboundMessage::ProcessingError(error.to_string()))
                            .await?;
                    }
                }
                OutboundMessage::Notification(notification) => {
                    if let Err(error) = lsp_send_notification(&mut writer, &notification).await {
                        rtx.send(InboundMessage::ProcessingError(error.to_string()))
                            .await?;
                    }
                }
            }
        }
        anyhow::Ok(())
    });

    // Sends responses from LSP's stdout to the editor
    let rtx = response_tx.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);

        loop {
            let response = match lsp_read_response(&mut reader).await {
                Err(err) => {
                    rtx.send(InboundMessage::ProcessingError(err.to_string()))
                        .await
                        .unwrap();
                    continue;
                }
                Ok(value) => {
                    log!("[lsp] incoming message: {value:?}");
                    value
                }
            };

            match process_response(&response) {
                Ok(message) => {
                    rtx.send(message).await.unwrap();
                }
                Err(err) => {
                    rtx.send(InboundMessage::ProcessingError(err.to_string()))
                        .await
                        .unwrap();
                }
            }
        }
    });

    // Sends responses from LSP's stderr to the editor
    let rtx = response_tx.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while let Ok(read) = reader.read_line(&mut line).await {
            if read > 0 {
                log!("[lsp] incoming stderr: {:?}", line);
                rtx.send(InboundMessage::ProcessingError(line.clone()))
                    .await
                    .unwrap();
            }
        }
    });

    Ok(LspClient {
        request_tx,
        response_rx,
        pending_responses: Default::default(),
    })
}

#[derive(Debug)]
pub struct LspClient {
    request_tx: mpsc::Sender<OutboundMessage>,
    response_rx: mpsc::Receiver<InboundMessage>,
    pending_responses: HashMap<i64, String>,
}

impl LspClient {
    pub async fn start() -> Result<Self> {
        start_lsp().await
    }

    pub async fn initialize(&mut self) -> Result<()> {
        let params = json!({
            "processId": process::id(),
            "clientInfo": {
                "name": "viron",
                "version": "0.1.0"
            },
            "capabilities": {
                "textDocument": {
                    "completion": {
                        "completionItem": {
                            "snippetSupport": true
                        }
                    },
                    "definition": {
                        "dynamicRegistration": true,
                        "linkSupport": false,
                    }
                }
            }
        });
        self.send_request("initialize", params).await?;
        _ = self.recv_response().await?;
        self.send_notification("initialized", json!({})).await?;
        Ok(())
    }

    pub async fn did_open(&mut self, file: &str, contents: &str) -> anyhow::Result<()> {
        log!("[lsp] did_open file: {}", file);
        let params = json!({
            "textDocument": {
                "uri": format!("file:///{}", file),
                "languageId": "rust",
                "version": 1,
                "text": contents,
            }
        });

        self.send_notification("textDocument/didOpen", params)
            .await?;

        Ok(())
    }

    pub async fn goto_definition(
        &mut self,
        file: &str,
        line: usize,
        character: usize,
    ) -> anyhow::Result<i64> {
        let params = json!({
            "textDocument": {
                "uri": format!("file:///{}", file),
            },
            "position": {
                "line": line,
                "character": character,
            }
        });

        Ok(self.send_request("textDocument/definition", params).await?)
    }

    pub async fn send_request(&mut self, method: &str, params: Value) -> Result<i64> {
        let req = Request::new(method, params);
        let id = req.id.clone();

        self.pending_responses.insert(id, method.to_string());
        self.request_tx.send(OutboundMessage::Request(req)).await?;

        log!("[lsp] request {id} sent: {:?}", method);
        Ok(id)
    }

    pub async fn send_notification(&mut self, method: &str, params: Value) -> Result<()> {
        self.request_tx
            .send(OutboundMessage::Notification(NotificationRequest {
                method: method.to_string(),
                params,
            }))
            .await?;
        Ok(())
    }

    pub async fn recv_response(
        &mut self,
    ) -> anyhow::Result<Option<(InboundMessage, Option<String>)>> {
        match self.response_rx.try_recv() {
            Ok(msg) => {
                if let InboundMessage::Message(msg) = &msg {
                    if let Some(method) = self.pending_responses.remove(&msg.id) {
                        return Ok(Some((InboundMessage::Message(msg.clone()), Some(method))));
                    }
                }
                Ok(Some((msg, None)))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

pub async fn lsp_send_request(
    stdin: &mut BufWriter<ChildStdin>,
    req: &Request,
) -> anyhow::Result<i64> {
    let id = req.id;
    let req = json!({
        "id": req.id,
        "jsonrpc": "2.0",
        "method": req.method,
        "params": req.params,
    });
    let body = serde_json::to_string(&req)?;
    let req = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
    stdin.write_all(req.as_bytes()).await?;
    stdin.flush().await?;

    Ok(id)
}

pub async fn lsp_send_notification(
    writer: &mut BufWriter<ChildStdin>,
    req: &NotificationRequest,
) -> Result<()> {
    let req = json!({
        "jsonrpc": "2.0",
        "method": req.method,
        "params": req.params,
    });
    let body = serde_json::to_string(&req)?;
    let req = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
    writer.write_all(req.as_bytes()).await?;

    Ok(())
}

pub async fn lsp_read_response(reader: &mut BufReader<ChildStdout>) -> Result<Value> {
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let length = line
        .strip_prefix("Content-Length: ")
        .context("Expected Content-length header")?
        .trim()
        .parse::<usize>()?;
    reader.read_line(&mut line).await?;

    let mut body = vec![0; length];
    reader.read_exact(&mut body).await?;

    let body = String::from_utf8(body)?;
    Ok(serde_json::from_str(&body)?)
}

pub fn get_error_message(error: &Value) -> Result<InboundMessage> {
    let code = error
        .get("code")
        .and_then(|s| s.as_i64())
        .context("Expected integer property - code")?;
    let message = error
        .get("message")
        .and_then(|s| s.as_str())
        .context("Expected string property - message")?
        .to_string();
    let data = error.get("data").cloned();
    Ok(InboundMessage::Error(ResponseError {
        code,
        message,
        data,
    }))
}

pub fn process_response(response: &Value) -> Result<InboundMessage> {
    if let Some(id) = response.get("id") {
        let id = id.as_i64().context("Expected id as integer")?;
        let result = response
            .get("result")
            .cloned()
            .context("Expcted property - result")?;
        Ok(InboundMessage::Message(ResponseMessage { id, result }))
    } else {
        let method = response
            .get("method")
            .and_then(|s| s.as_str())
            .context("Expected string property - method")?
            .to_string();
        let params = response
            .get("params")
            .cloned()
            .context("Expected property - params")?;
        Ok(InboundMessage::Notification(Notification {
            method,
            params,
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_start_lsp() {
        let mut client = LspClient::start().await.unwrap();
        client.initialize().await.unwrap();
    }
}
