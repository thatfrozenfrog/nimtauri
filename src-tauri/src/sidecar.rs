use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt,
};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

const PROTOCOL_VERSION: u64 = 1;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendStatus {
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_error: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BackendError {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl BackendError {
    fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            data: None,
        }
    }
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    id: String,
    ok: bool,
    #[serde(default)]
    result: Value,
    #[serde(default)]
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: String,
    message: String,
    #[serde(default)]
    data: Option<Value>,
}

impl From<RpcError> for BackendError {
    fn from(error: RpcError) -> Self {
        Self {
            code: error.code,
            message: error.message,
            data: error.data,
        }
    }
}

type PendingRequests = HashMap<String, (u64, oneshot::Sender<Result<Value, BackendError>>)>;

fn ready_status(data: &Value) -> Result<BackendStatus, BackendError> {
    let protocol = data.get("protocolVersion").and_then(Value::as_u64);
    if protocol != Some(PROTOCOL_VERSION) {
        return Err(BackendError::new(
            "PROTOCOL_VERSION_MISMATCH",
            format!(
                "expected protocol version {PROTOCOL_VERSION}, received {}",
                protocol
                    .map(|version| version.to_string())
                    .unwrap_or_else(|| "none".into())
            ),
        ));
    }

    Ok(BackendStatus {
        state: "ready".into(),
        version: data
            .get("backendVersion")
            .and_then(Value::as_str)
            .map(str::to_owned),
        protocol,
        last_error: None,
    })
}

fn resolve_response(pending: &mut PendingRequests, generation: u64, response: RpcResponse) {
    if let Some((response_generation, sender)) = pending.remove(&response.id) {
        if response_generation != generation {
            return;
        }
        let result = if response.ok {
            Ok(response.result)
        } else {
            Err(response.error.map(BackendError::from).unwrap_or_else(|| {
                BackendError::new("PROTOCOL_ERROR", "backend returned an unspecified error")
            }))
        };
        let _ = sender.send(result);
    }
}

fn reject_generation(pending: &mut PendingRequests, generation: u64) {
    let requests = std::mem::take(pending);
    for (id, (request_generation, sender)) in requests {
        if request_generation == generation {
            let _ = sender.send(Err(BackendError::new(
                "PROCESS_TERMINATED",
                "the Nim sidecar stopped before responding",
            )));
        } else {
            pending.insert(id, (request_generation, sender));
        }
    }
}

struct Inner {
    app: AppHandle,
    child: Mutex<Option<(u64, CommandChild)>>,
    pending: Mutex<PendingRequests>,
    status: Mutex<BackendStatus>,
    generation: AtomicU64,
    restart_lock: Mutex<()>,
}

#[derive(Clone)]
pub struct SidecarManager {
    inner: Arc<Inner>,
}

impl SidecarManager {
    pub fn new(app: AppHandle) -> Self {
        Self {
            inner: Arc::new(Inner {
                app,
                child: Mutex::new(None),
                pending: Mutex::new(HashMap::new()),
                status: Mutex::new(BackendStatus {
                    state: "starting".into(),
                    version: None,
                    protocol: None,
                    last_error: None,
                }),
                generation: AtomicU64::new(0),
                restart_lock: Mutex::new(()),
            }),
        }
    }

    pub async fn status(&self) -> BackendStatus {
        self.inner.status.lock().await.clone()
    }

    pub async fn start(&self) -> Result<(), BackendError> {
        let generation = self.inner.generation.fetch_add(1, Ordering::SeqCst) + 1;

        {
            let mut status = self.inner.status.lock().await;
            *status = BackendStatus {
                state: "starting".into(),
                version: None,
                protocol: None,
                last_error: None,
            };
        }

        let command = match self.inner.app.shell().sidecar("nim-backend") {
            Ok(command) => command,
            Err(error) => {
                let error = BackendError::new("SIDECAR_START_FAILED", error.to_string());
                self.mark_failed(generation, error.clone()).await;
                return Err(error);
            }
        };

        let (mut events, child) = match command.spawn() {
            Ok(process) => process,
            Err(error) => {
                let error = BackendError::new("SIDECAR_START_FAILED", error.to_string());
                self.mark_failed(generation, error.clone()).await;
                return Err(error);
            }
        };
        *self.inner.child.lock().await = Some((generation, child));

        let manager = self.clone();
        tauri::async_runtime::spawn(async move {
            let mut stdout_buffer = Vec::<u8>::new();

            while let Some(event) = events.recv().await {
                match event {
                    CommandEvent::Stdout(chunk) => {
                        stdout_buffer.extend_from_slice(&chunk);
                        while let Some(position) =
                            stdout_buffer.iter().position(|byte| *byte == b'\n')
                        {
                            let line = stdout_buffer.drain(..=position).collect::<Vec<_>>();
                            let line = String::from_utf8_lossy(&line).trim().to_owned();
                            if !line.is_empty() {
                                manager.handle_line(generation, &line).await;
                            }
                        }
                    }
                    CommandEvent::Stderr(chunk) => {
                        eprint!("{}", String::from_utf8_lossy(&chunk));
                    }
                    CommandEvent::Error(error) => {
                        manager
                            .fail(generation, BackendError::new("SIDECAR_IO_ERROR", error))
                            .await;
                    }
                    CommandEvent::Terminated(payload) => {
                        manager.stop_child(generation).await;
                        manager
                            .mark_failed(
                                generation,
                                BackendError::new(
                                    "PROCESS_TERMINATED",
                                    format!("sidecar terminated with {:?}", payload.code),
                                ),
                            )
                            .await;
                        break;
                    }
                    _ => {}
                }
            }
        });

        let manager = self.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_secs(5)).await;
            if !manager.is_current(generation) {
                return;
            }

            if manager.inner.status.lock().await.state == "starting" {
                manager
                    .fail(
                        generation,
                        BackendError::new("SIDECAR_START_TIMEOUT", "backend readiness timeout"),
                    )
                    .await;
            }
        });

        Ok(())
    }

    fn is_current(&self, generation: u64) -> bool {
        self.inner.generation.load(Ordering::SeqCst) == generation
    }

    async fn handle_line(&self, generation: u64, line: &str) {
        if !self.is_current(generation) {
            return;
        }

        let value: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(error) => {
                self.fail(
                    generation,
                    BackendError::new(
                        "PROTOCOL_ERROR",
                        format!("invalid JSON from sidecar: {error}"),
                    ),
                )
                .await;
                return;
            }
        };

        if let Some(event) = value.get("event").and_then(Value::as_str) {
            if event == "backend.ready" {
                let data = value.get("data").cloned().unwrap_or(Value::Null);
                let status = match ready_status(&data) {
                    Ok(status) => status,
                    Err(error) => {
                        self.fail(generation, error).await;
                        return;
                    }
                };

                let mut current_status = self.inner.status.lock().await;
                if !self.is_current(generation) {
                    return;
                }
                *current_status = status;
            }
            let _ = self.inner.app.emit("nim://event", value);
            return;
        }

        let response: RpcResponse = match serde_json::from_value(value) {
            Ok(response) => response,
            Err(error) => {
                self.fail(
                    generation,
                    BackendError::new(
                        "PROTOCOL_ERROR",
                        format!("invalid response shape from sidecar: {error}"),
                    ),
                )
                .await;
                return;
            }
        };

        resolve_response(&mut *self.inner.pending.lock().await, generation, response);
    }

    async fn stop_child(&self, generation: u64) {
        let child = {
            let mut child = self.inner.child.lock().await;
            match child.as_ref() {
                Some((child_generation, _)) if *child_generation == generation => {
                    child.take().map(|(_, child)| child)
                }
                _ => None,
            }
        };
        if let Some(child) = child {
            let _ = child.kill();
        }
    }

    async fn fail(&self, generation: u64, error: BackendError) {
        self.stop_child(generation).await;
        self.mark_failed(generation, error).await;
    }

    async fn mark_failed(&self, generation: u64, error: BackendError) {
        if !self.is_current(generation) {
            return;
        }

        {
            let mut status = self.inner.status.lock().await;
            if !self.is_current(generation) {
                return;
            }
            *status = BackendStatus {
                state: "failed".into(),
                version: None,
                protocol: None,
                last_error: Some(error.message.clone()),
            };
        }

        self.reject_pending(generation).await;
        let _ = self.inner.app.emit("nim://failed", error);
    }

    async fn reject_pending(&self, generation: u64) {
        let mut pending = self.inner.pending.lock().await;
        reject_generation(&mut pending, generation);
    }

    async fn call_with_timeout(
        &self,
        method: String,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, BackendError> {
        if self.status().await.state != "ready" {
            return Err(BackendError::new(
                "BACKEND_NOT_READY",
                "the Nim sidecar is not ready",
            ));
        }

        let id = Uuid::new_v4().to_string();
        let request = json!({ "id": id, "method": method, "params": params });
        let encoded = serde_json::to_vec(&request)
            .map_err(|error| BackendError::new("REQUEST_ENCODE_ERROR", error.to_string()))?;
        let (sender, receiver) = oneshot::channel();
        let generation = self.inner.generation.load(Ordering::SeqCst);
        self.inner
            .pending
            .lock()
            .await
            .insert(id.clone(), (generation, sender));

        let write_result = {
            let mut child = self.inner.child.lock().await;
            match child.as_mut() {
                Some((child_generation, child)) if *child_generation == generation => child
                    .write([encoded, vec![b'\n']].concat().as_slice())
                    .map_err(|error| BackendError::new("SIDECAR_IO_ERROR", error.to_string())),
                Some(_) | None => Err(BackendError::new(
                    "PROCESS_TERMINATED",
                    "the Nim sidecar is not running",
                )),
            }
        };

        if let Err(error) = write_result {
            self.inner.pending.lock().await.remove(&id);
            return Err(error);
        }

        match tokio::time::timeout(timeout, receiver).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(BackendError::new(
                "PROCESS_TERMINATED",
                "the Nim sidecar stopped before responding",
            )),
            Err(_) => {
                self.inner.pending.lock().await.remove(&id);
                Err(BackendError::new(
                    "REQUEST_TIMEOUT",
                    "the Nim sidecar did not respond in time",
                ))
            }
        }
    }

    pub async fn call(&self, method: String, params: Value) -> Result<Value, BackendError> {
        self.call_with_timeout(method, params, REQUEST_TIMEOUT)
            .await
    }

    pub async fn restart(&self) -> Result<BackendStatus, BackendError> {
        let _restart_guard = self.inner.restart_lock.lock().await;
        let previous_generation = self.inner.generation.load(Ordering::SeqCst);

        self.stop_child(previous_generation).await;
        self.reject_pending(previous_generation).await;
        self.start().await?;

        for _ in 0..50 {
            let status = self.status().await;
            if status.state == "ready" {
                return Ok(status);
            }
            if status.state == "failed" {
                return Err(BackendError::new(
                    "SIDECAR_START_FAILED",
                    status.last_error.unwrap_or_else(|| "sidecar failed".into()),
                ));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(BackendError::new(
            "SIDECAR_START_TIMEOUT",
            "backend readiness timeout",
        ))
    }

    pub async fn shutdown(&self) {
        if self.status().await.state == "ready" {
            let _ = self
                .call_with_timeout("system.shutdown".into(), json!({}), SHUTDOWN_TIMEOUT)
                .await;
        }

        let generation = self.inner.generation.fetch_add(1, Ordering::SeqCst);
        self.stop_child(generation).await;

        for (_, (_, sender)) in self.inner.pending.lock().await.drain() {
            let _ = sender.send(Err(BackendError::new(
                "PROCESS_TERMINATED",
                "the Nim sidecar is shutting down",
            )));
        }

        *self.inner.status.lock().await = BackendStatus {
            state: "stopped".into(),
            version: None,
            protocol: None,
            last_error: None,
        };
    }
}

#[tauri::command]
pub async fn backend_call(
    manager: tauri::State<'_, SidecarManager>,
    method: String,
    params: Value,
) -> Result<Value, BackendError> {
    manager.call(method, params).await
}

#[tauri::command]
pub async fn backend_status(
    manager: tauri::State<'_, SidecarManager>,
) -> Result<BackendStatus, BackendError> {
    Ok(manager.status().await)
}

#[tauri::command]
pub async fn backend_restart(
    manager: tauri::State<'_, SidecarManager>,
) -> Result<BackendStatus, BackendError> {
    manager.restart().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_incompatible_protocol_versions() {
        let error = ready_status(&json!({ "protocolVersion": 2 })).unwrap_err();

        assert_eq!(error.code, "PROTOCOL_VERSION_MISMATCH");
        assert!(error.message.contains("expected protocol version 1"));
    }

    #[test]
    fn routes_concurrent_responses_by_request_id() {
        let mut pending = PendingRequests::new();
        let (first_sender, mut first_receiver) = oneshot::channel();
        let (second_sender, mut second_receiver) = oneshot::channel();
        pending.insert("first".into(), (1, first_sender));
        pending.insert("second".into(), (1, second_sender));

        resolve_response(
            &mut pending,
            1,
            serde_json::from_str(r#"{"id":"second","ok":true,"result":{"value":2}}"#).unwrap(),
        );
        resolve_response(
            &mut pending,
            1,
            serde_json::from_str(r#"{"id":"first","ok":true,"result":{"value":1}}"#).unwrap(),
        );

        assert_eq!(
            second_receiver.try_recv().unwrap().unwrap()["value"],
            Value::from(2)
        );
        assert_eq!(
            first_receiver.try_recv().unwrap().unwrap()["value"],
            Value::from(1)
        );
    }

    #[test]
    fn only_rejects_requests_from_the_stopped_generation() {
        let mut pending = PendingRequests::new();
        let (old_sender, mut old_receiver) = oneshot::channel();
        let (current_sender, mut current_receiver) = oneshot::channel();
        pending.insert("old".into(), (1, old_sender));
        pending.insert("current".into(), (2, current_sender));

        reject_generation(&mut pending, 1);

        assert_eq!(
            old_receiver.try_recv().unwrap().unwrap_err().code,
            "PROCESS_TERMINATED"
        );
        assert!(current_receiver.try_recv().is_err());
    }
}
