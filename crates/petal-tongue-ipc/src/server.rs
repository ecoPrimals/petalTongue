//! IPC server implementation
//!
//! Unix domain socket server for handling commands from other instances or CLI tools.

use crate::protocol::{IpcCommand, IpcResponse, InstanceStatus};
use petal_tongue_core::{Instance, InstanceId};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;

/// IPC server for handling commands
pub struct IpcServer {
    /// Instance ID this server represents
    instance_id: InstanceId,

    /// Path to Unix domain socket
    socket_path: PathBuf,

    /// Receiver for commands from the listener task
    command_rx: mpsc::UnboundedReceiver<(IpcCommand, mpsc::UnboundedSender<IpcResponse>)>,

    /// Sender for the listener task (for shutdown)
    _shutdown_tx: mpsc::UnboundedSender<()>,
}

impl IpcServer {
    /// Start a new IPC server
    ///
    /// Creates a Unix domain socket at the instance's socket path and starts
    /// listening for connections in a background task.
    ///
    /// # Errors
    ///
    /// Returns error if socket cannot be created or bound
    pub async fn start(instance: &Instance) -> Result<Self, IpcServerError> {
        let socket_path = instance.socket_path.clone();
        let instance_id = instance.id.clone();

        // Remove old socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path).map_err(|e| {
                IpcServerError::SocketError(format!("Failed to remove old socket: {}", e))
            })?;
        }

        // Create Unix listener
        let listener = UnixListener::bind(&socket_path).map_err(|e| {
            IpcServerError::SocketError(format!("Failed to bind socket: {}", e))
        })?;

        tracing::info!("IPC server started at: {}", socket_path.display());

        // Create channels for communication
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();

        // Spawn listener task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Check for shutdown signal
                    _ = shutdown_rx.recv() => {
                        tracing::info!("IPC server shutting down");
                        break;
                    }

                    // Accept new connections
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _)) => {
                                let command_tx = command_tx.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = handle_connection(stream, command_tx).await {
                                        tracing::warn!("Connection error: {}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("Accept error: {}", e);
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            instance_id,
            socket_path,
            command_rx,
            _shutdown_tx: shutdown_tx,
        })
    }

    /// Receive the next command
    ///
    /// Returns the command and a sender to send the response back.
    pub async fn recv_command(
        &mut self,
    ) -> Option<(IpcCommand, mpsc::UnboundedSender<IpcResponse>)> {
        self.command_rx.recv().await
    }

    /// Get the socket path
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Get the instance ID
    #[must_use]
    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        // Clean up socket file
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
                tracing::warn!("Failed to remove socket file: {}", e);
            }
        }
    }
}

/// Handle a single IPC connection
async fn handle_connection(
    stream: UnixStream,
    command_tx: mpsc::UnboundedSender<(IpcCommand, mpsc::UnboundedSender<IpcResponse>)>,
) -> Result<(), IpcServerError> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Read command (JSON line)
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .map_err(|e| IpcServerError::IoError(format!("Failed to read command: {}", e)))?;

    // Parse command
    let command: IpcCommand = serde_json::from_str(&line)
        .map_err(|e| IpcServerError::ParseError(format!("Failed to parse command: {}", e)))?;

    tracing::debug!("Received IPC command: {}", command.name());

    // Create response channel
    let (response_tx, mut response_rx) = mpsc::unbounded_channel();

    // Send command to main task
    command_tx
        .send((command, response_tx))
        .map_err(|_| IpcServerError::ChannelClosed)?;

    // Wait for response
    let response = response_rx
        .recv()
        .await
        .ok_or(IpcServerError::ChannelClosed)?;

    // Send response (JSON line)
    let response_json = serde_json::to_string(&response)
        .map_err(|e| IpcServerError::SerializeError(format!("Failed to serialize response: {}", e)))?;

    writer
        .write_all(response_json.as_bytes())
        .await
        .map_err(|e| IpcServerError::IoError(format!("Failed to write response: {}", e)))?;

    writer
        .write_all(b"\n")
        .await
        .map_err(|e| IpcServerError::IoError(format!("Failed to write newline: {}", e)))?;

    writer
        .flush()
        .await
        .map_err(|e| IpcServerError::IoError(format!("Failed to flush: {}", e)))?;

    Ok(())
}

/// Errors that can occur in the IPC server
#[derive(Debug, Error)]
pub enum IpcServerError {
    /// Socket error
    #[error("Socket error: {0}")]
    SocketError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Serialize error
    #[error("Serialize error: {0}")]
    SerializeError(String),

    /// Channel closed
    #[error("Channel closed")]
    ChannelClosed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::Instance;

    #[tokio::test]
    async fn test_server_creation() {
        let instance_id = InstanceId::new();
        let instance = Instance::new(instance_id, Some("test".to_string())).unwrap();

        let server = IpcServer::start(&instance).await;
        assert!(server.is_ok());

        let server = server.unwrap();
        assert_eq!(server.instance_id(), &instance.id);
        assert!(server.socket_path().exists());
    }
}

