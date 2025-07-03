use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process;
use std::sync::Arc;
use std::{
    process::Stdio,
    sync::atomic::{self, AtomicUsize},
};
use tokio::io::AsyncReadExt;
use tokio::process::Child;
use tokio::sync::Mutex;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{ChildStdin, ChildStdout, Command},
    sync::mpsc,
};

use crate::service::lsp::types::{LogMessageParams, ShowMessageParams};
use crate::utils;
pub use types::TextDocumentPublishDiagnostics;

pub mod types;

static ID: AtomicUsize = AtomicUsize::new(1);
const CHANNEL_SIZE: usize = 32;

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
    Notification(NotificationKind),
    UnknownNotification(Notification),
    Error(ResponseError),
    ProcessingError(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum NotificationKind {
    PublishDiagnostics(TextDocumentPublishDiagnostics),
    ShowMessage(ShowMessageParams),
    LogMessage(LogMessageParams),
}

#[derive(Debug)]
pub struct LspClient {
    request_sender: mpsc::Sender<OutboundMessage>,
    response_receiver: mpsc::Receiver<InboundMessage>,
    pending_responses: HashMap<i64, String>,
    process: Arc<Mutex<Option<Child>>>,
    is_initialized: bool,
}

impl LspClient {
    pub async fn start(server: &str, args: &[&str]) -> Result<Self> {
        let mut child = Command::new(server)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;
        let stderr = child.stderr.take().context("Failed to get stderr")?;

        let (request_sender, mut request_receiver) = mpsc::channel::<OutboundMessage>(CHANNEL_SIZE);
        let (response_sender, response_receiver) = mpsc::channel::<InboundMessage>(CHANNEL_SIZE);

        // Send requests from editor into LSP's stdin
        let sender = response_sender.clone();
        tokio::spawn(async move {
            let mut writer = BufWriter::new(stdin);
            while let Some(message) = request_receiver.recv().await {
                match message {
                    OutboundMessage::Request(request) => {
                        if let Err(error) = lsp_send_request(&mut writer, &request).await {
                            sender
                                .send(InboundMessage::ProcessingError(error.to_string()))
                                .await?;
                        }
                    }
                    OutboundMessage::Notification(notification) => {
                        if let Err(error) = lsp_send_notification(&mut writer, &notification).await
                        {
                            sender
                                .send(InboundMessage::ProcessingError(error.to_string()))
                                .await?;
                        }
                    }
                }
            }
            anyhow::Ok(())
        });

        // Sends responses from LSP's stdout to the editor
        let sender = response_sender.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);

            loop {
                let response = match lsp_read_response(&mut reader).await {
                    Err(err) => {
                        sender
                            .send(InboundMessage::ProcessingError(err.to_string()))
                            .await
                            .unwrap();
                        continue;
                    }
                    Ok(value) => value,
                };

                match process_response(&response) {
                    Ok(message) => {
                        sender.send(message).await.unwrap();
                    }
                    Err(err) => {
                        sender
                            .send(InboundMessage::ProcessingError(err.to_string()))
                            .await
                            .unwrap();
                    }
                }
            }
        });

        // Sends responses from LSP's stderr to the editor
        let sender = response_sender.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while let Ok(read) = reader.read_line(&mut line).await {
                if read > 0 {
                    sender
                        .send(InboundMessage::ProcessingError(line.clone()))
                        .await
                        .unwrap();
                }
            }
        });

        Ok(LspClient {
            request_sender,
            response_receiver,
            pending_responses: Default::default(),
            process: Arc::new(Mutex::new(Some(child))),
            is_initialized: false,
        })
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
        _ = self.receive_response().await?;
        self.send_notification("initialized", json!({})).await?;
        Ok(())
    }

    pub async fn did_open(&mut self, file: &str, contents: &str) -> anyhow::Result<()> {
        let params = json!({
            "textDocument": {
                "uri": format!("file://{}", utils::absolutize(file)?.to_string_lossy()),
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
                "uri": format!("file://{}", utils::absolutize(file)?.to_string_lossy()),
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
        self.request_sender
            .send(OutboundMessage::Request(req))
            .await?;
        Ok(id)
    }

    pub async fn send_notification(&mut self, method: &str, params: Value) -> Result<()> {
        self.request_sender
            .send(OutboundMessage::Notification(NotificationRequest {
                method: method.to_string(),
                params,
            }))
            .await?;
        Ok(())
    }

    pub async fn receive_response(
        &mut self,
    ) -> anyhow::Result<Option<(InboundMessage, Option<String>)>> {
        match self.response_receiver.try_recv() {
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

    
}

fn parse_notification(method: &str, params: &Value) -> Result<Option<NotificationKind>> {
    match method {
        "textDocument/publishDiagnostics" => Ok(Some(NotificationKind::PublishDiagnostics(
            serde_json::from_value(params.clone())?,
        ))),
        "window/showMessage" => Ok(Some(NotificationKind::ShowMessage(serde_json::from_value(
            params.clone(),
        )?))),
        "window/logMessage" => Ok(Some(NotificationKind::LogMessage(serde_json::from_value(
            params.clone(),
        )?))),
        _ => Ok(None),
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

        match parse_notification(&method, &params)? {
            Some(notification) => Ok(InboundMessage::Notification(notification)),
            None => Ok(InboundMessage::UnknownNotification(Notification {
                method,
                params,
            })),
        }
    }
}
