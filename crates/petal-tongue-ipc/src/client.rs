// SPDX-License-Identifier: AGPL-3.0-only
//! IPC client implementation
//!
//! Client for connecting to and communicating with petalTongue instances.

use crate::protocol::{IpcCommand, IpcResponse};
use petal_tongue_core::InstanceId;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// IPC client for communicating with instances
pub struct IpcClient {
    /// Path to the socket
    socket_path: PathBuf,
}

impl IpcClient {
    /// Connect to an instance by ID
    ///
    /// # Errors
    ///
    /// Returns error if socket cannot be found or connection fails
    pub fn new(instance_id: &InstanceId) -> Result<Self, IpcClientError> {
        let socket_path = get_socket_path(instance_id)?;

        if !socket_path.exists() {
            return Err(IpcClientError::SocketNotFound(socket_path));
        }

        Ok(Self { socket_path })
    }

    /// Connect to an instance by socket path
    #[must_use]
    pub const fn from_socket_path(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Send a command and wait for response
    ///
    /// # Errors
    ///
    /// Returns error if connection fails or communication error occurs
    pub async fn send(&self, command: IpcCommand) -> Result<IpcResponse, IpcClientError> {
        // Connect to socket
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| IpcClientError::ConnectionError(format!("Failed to connect: {e}")))?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Serialize and send command
        let command_json = serde_json::to_string(&command)
            .map_err(|e| IpcClientError::SerializeError(format!("Failed to serialize: {e}")))?;

        writer
            .write_all(command_json.as_bytes())
            .await
            .map_err(|e| IpcClientError::IoError(format!("Failed to write: {e}")))?;

        writer
            .write_all(b"\n")
            .await
            .map_err(|e| IpcClientError::IoError(format!("Failed to write newline: {e}")))?;

        writer
            .flush()
            .await
            .map_err(|e| IpcClientError::IoError(format!("Failed to flush: {e}")))?;

        // Read response
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .map_err(|e| IpcClientError::IoError(format!("Failed to read response: {e}")))?;

        // Parse response
        let response: IpcResponse = serde_json::from_str(&line)
            .map_err(|e| IpcClientError::ParseError(format!("Failed to parse response: {e}")))?;

        Ok(response)
    }

    /// Ping the instance
    ///
    /// # Errors
    ///
    /// Returns error if ping fails
    pub async fn ping(&self) -> Result<(), IpcClientError> {
        let response = self.send(IpcCommand::Ping).await?;
        match response {
            IpcResponse::Pong => Ok(()),
            IpcResponse::Error { message } => Err(IpcClientError::ServerError(message)),
            _ => Err(IpcClientError::UnexpectedResponse),
        }
    }

    /// Get socket path
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

/// Errors that can occur in the IPC client
#[derive(Debug, Error)]
pub enum IpcClientError {
    /// Socket not found
    #[error("Socket not found: {0}")]
    SocketNotFound(PathBuf),

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Serialize error
    #[error("Serialize error: {0}")]
    SerializeError(String),

    /// Server returned error
    #[error("Server error: {0}")]
    ServerError(String),

    /// Unexpected response
    #[error("Unexpected response from server")]
    UnexpectedResponse,

    /// Directory error
    #[error("Directory error: {0}")]
    DirectoryError(String),
}

/// Get socket path for an instance
#[expect(
    clippy::unnecessary_wraps,
    reason = "Ok wrapper for early-return path consistency"
)]
fn get_socket_path(instance_id: &InstanceId) -> Result<PathBuf, IpcClientError> {
    use petal_tongue_core::constants::APP_DIR_NAME;

    // Try /run/user/{uid}/{app_dir} first
    if let Ok(uid) = std::env::var("UID") {
        let run_dir = PathBuf::from(format!("/run/user/{uid}/{APP_DIR_NAME}"));
        if run_dir.exists() {
            return Ok(run_dir.join(format!("{}.sock", instance_id.as_str())));
        }
    }

    // Fall back to /tmp/{app_dir}
    Ok(PathBuf::from(format!(
        "/tmp/{}/{}.sock",
        APP_DIR_NAME,
        instance_id.as_str()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_client_creation_socket_not_found() {
        let instance_id = InstanceId::new();
        let result = IpcClient::new(&instance_id);

        // Will fail since socket doesn't exist, but tests the code path
        assert!(result.is_err());
        if let Err(IpcClientError::SocketNotFound(p)) = result {
            assert!(p.to_string_lossy().contains(&instance_id.as_str()));
        }
    }

    #[test]
    fn test_socket_path() {
        let instance_id = InstanceId::new();
        let path = get_socket_path(&instance_id).expect("path");
        assert!(path.to_string_lossy().contains(&instance_id.as_str()));
    }

    #[test]
    fn test_from_socket_path() {
        let path = PathBuf::from("/tmp/test-socket.sock");
        let client = IpcClient::from_socket_path(path.clone());
        assert_eq!(client.socket_path(), path);
    }

    #[test]
    fn test_socket_path_getter() {
        let path = PathBuf::from("/var/run/petal.sock");
        let client = IpcClient::from_socket_path(path);
        assert_eq!(client.socket_path(), Path::new("/var/run/petal.sock"));
    }

    #[test]
    fn test_ipc_client_error_display() {
        let err = IpcClientError::SocketNotFound(PathBuf::from("/tmp/x.sock"));
        assert!(format!("{err}").contains("Socket not found"));

        let err = IpcClientError::ConnectionError("msg".into());
        assert!(format!("{err}").contains("Connection error"));

        let err = IpcClientError::IoError("msg".into());
        assert!(format!("{err}").contains("IO error"));

        let err = IpcClientError::ParseError("msg".into());
        assert!(format!("{err}").contains("Parse error"));

        let err = IpcClientError::SerializeError("msg".into());
        assert!(format!("{err}").contains("Serialize error"));

        let err = IpcClientError::ServerError("msg".into());
        assert!(format!("{err}").contains("Server error"));

        let err = IpcClientError::UnexpectedResponse;
        assert!(!format!("{err}").is_empty());

        let err = IpcClientError::DirectoryError("msg".into());
        assert!(format!("{err}").contains("Directory error"));
    }

    #[tokio::test]
    async fn test_ping_nonexistent_socket() {
        let path = PathBuf::from("/tmp/nonexistent-ipc-ping-99999.sock");
        let client = IpcClient::from_socket_path(path);
        let result = client.ping().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_nonexistent_socket() {
        let path = PathBuf::from("/tmp/nonexistent-ipc-send-99999.sock");
        let client = IpcClient::from_socket_path(path);
        let result = client.send(IpcCommand::Ping).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_get_socket_path_format() {
        let instance_id = InstanceId::new();
        let path = get_socket_path(&instance_id).expect("path");
        let path_str = path.to_string_lossy();
        assert!(path_str.contains(&instance_id.as_str()));
        assert!(path_str.ends_with(".sock"));
        assert!(path_str.contains("petaltongue"));
    }
}
