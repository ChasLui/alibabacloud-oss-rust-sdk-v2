use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use serde_json;
use wait_timeout::ChildExt;

use crate::credential::{
    new_credentials_fetcher_provider, Credentials, CredentialsFetcher, CredentialsProvider,
};

/// The `ProcessCredentialsProvider` struct represents a credentials provider
/// that fetches credentials by executing a process command and parsing the
/// output.
pub struct ProcessCredentialsProvider {
    timeout: Duration,
    args: Vec<String>,
}

impl ProcessCredentialsProvider {
    /// Creates a new instance of `ProcessCredentialsProvider`.
    /// It takes a command string and an optional timeout duration as input.
    /// If the command string is empty, an empty vector is used for the command
    /// arguments. If the timeout duration is not provided, a default
    /// timeout of 15 seconds is used.
    pub fn new(command: &str, opt_timeout: Option<Duration>) -> Self {
        let args = if !command.is_empty() {
            vec![command.to_owned()]
        } else {
            Vec::new()
        };
        ProcessCredentialsProvider {
            timeout: opt_timeout.unwrap_or(Duration::from_secs(15)),
            args,
        }
    }

    /// Executes the process command and returns the output as a byte vector.
    /// If the command is empty, an error is returned.
    /// The command is executed with a timeout, and if it succeeds, the output
    /// is returned. If the command times out, an error is returned.
    async fn execute_process(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        if self.args.is_empty() {
            return Err("Command must not be empty".into());
        }

        // Execute the command with a timeout
        let mut child = Command::new(if cfg!(target_os = "windows") {
            "cmd.exe"
        } else {
            "sh"
        })
        .arg(if cfg!(target_os = "windows") {
            "/C"
        } else {
            "-c"
        })
        .arg(self.args.join(" "))
        .envs(std::env::vars())
        .stdout(Stdio::piped())
        .spawn()?;

        // Wait for timeout
        match child
            .wait_timeout(self.timeout)
            .expect("Failed to wait for child process")
        {
            Some(status) => {
                if status.success() {
                    // get success output
                    let output = child.wait_with_output()?;
                    Ok(output.stdout)
                } else {
                    Err(format!("Error running command: {}", status).into())
                }
            }
            None => {
                // Kill timeout process
                child.kill()?;
                child.wait()?; // Make sure resources are cleaned up
                Err("Command timeout".into())
            }
        }
    }
}

#[async_trait::async_trait]
impl CredentialsProvider for ProcessCredentialsProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    /// Asynchronously fetches the credentials by calling the
    /// `fetch_credentials` method. It returns a `Result` containing the
    /// parsed `Credentials` struct or an error.
    async fn get_credentials(
        &self,
    ) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
        let output = self.execute_process().await?;
        let result: Credentials = serde_json::from_slice(&output)?;

        if !result.has_keys() {
            return Err("Missing AccessKeyId or AccessKeySecret in process output".into());
        }
        Ok(result)
    }
}

pub fn new_process_credentials_fetcher_provider(
    command: &str,
    timeout: Option<Duration>,
    refresh_duration: Option<Duration>,
    expired_factor: Option<f64>,
) -> impl CredentialsProvider {
    struct ProcessCredentialsFetcher {
        command: String,
        timeout: Option<Duration>,
    }

    impl ProcessCredentialsFetcher {
        pub fn new(command: &str, timeout: Option<Duration>) -> Self {
            Self {
                command: command.into(),
                timeout,
            }
        }
    }

    #[async_trait::async_trait]
    impl CredentialsFetcher for ProcessCredentialsFetcher {
        async fn fetch(&self) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
            ProcessCredentialsProvider::new(&self.command, self.timeout)
                .get_credentials()
                .await
        }
    }

    new_credentials_fetcher_provider(
        Arc::new(ProcessCredentialsFetcher::new(command, timeout)),
        refresh_duration,
        expired_factor,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_command() {
        let provider = ProcessCredentialsProvider::new("my_command", None);
        assert_eq!(provider.timeout, Duration::from_secs(15));
        assert_eq!(provider.args, vec!["my_command"]);
    }

    #[test]
    fn test_new_without_command() {
        let provider = ProcessCredentialsProvider::new("", Some(Duration::from_secs(30)));
        assert_eq!(provider.timeout, Duration::from_secs(30));
        assert_eq!(provider.args, Vec::<String>::new());
    }

    #[tokio::test]
    #[cfg(target_os = "linux")]
    async fn test_execute_command() {
        let provider = ProcessCredentialsProvider::new("echo hello", None);
        let output = provider.execute_process().await;
        assert!(output.is_ok());
    }

    #[tokio::test]
    #[cfg(target_os = "linux")]
    async fn test_get_credentials_success() {
        let credentials = Credentials {
            access_key_id: "access_key_id".into(),
            access_key_secret: "access_key_secret".into(),
            ..Default::default()
        };
        let serialized_credentials = serde_json::to_string(&credentials).unwrap();

        let provider = ProcessCredentialsProvider::new(
            format!("echo '{}'", serialized_credentials).as_str(),
            None,
        );

        let fetched_credentials = provider.get_credentials().await.ok().unwrap();
        assert_eq!(fetched_credentials, credentials);
    }

    #[tokio::test]
    #[cfg(target_os = "linux")]
    async fn test_get_credentials_missing_keys() {
        let credentials = Credentials {
            access_key_id: "".into(),
            access_key_secret: "access_key_secret".into(),
            ..Default::default()
        };
        let serialized_credentials = serde_json::to_string(&credentials).unwrap();

        let provider = ProcessCredentialsProvider::new(
            format!(r#"echo "{}""#, serialized_credentials.replace('"', r#"\""#)).as_str(),
            None,
        );

        let result = provider.get_credentials().await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Missing AccessKeyId or AccessKeySecret in process output"
        );
    }

    #[tokio::test]
    async fn test_get_credentials_error_command_not_found() {
        let provider = ProcessCredentialsProvider::new("a_non_existent_command", None);

        let result = provider.get_credentials().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .starts_with("Error running command"));
    }

    #[tokio::test]
    async fn test_get_credentials_error_empty_command() {
        let provider = ProcessCredentialsProvider::new("", None);

        let result = provider.get_credentials().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .starts_with("Command must not be empty"));
    }

    #[tokio::test]
    async fn test_get_credentials_error_timeout() {
        let credentials = Credentials {
            access_key_id: "access_key_id".into(),
            access_key_secret: "access_key_secret".into(),
            ..Default::default()
        };
        let serialized_credentials = serde_json::to_string(&credentials).unwrap();

        let provider = ProcessCredentialsProvider::new(
            format!("sleep 10; echo '{}'", serialized_credentials).as_str(),
            Some(Duration::from_nanos(1)),
        );

        let result = provider.get_credentials().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .starts_with("Command timeout"));
    }

    #[tokio::test]
    #[cfg(target_os = "linux")]
    async fn test_new_process_credentials_provider() {
        let credentials = Credentials {
            access_key_id: "access_key_id".into(),
            access_key_secret: "access_key_secret".into(),
            ..Default::default()
        };
        let serialized_credentials = serde_json::to_string(&credentials).unwrap();

        let provider = new_process_credentials_fetcher_provider(
            format!("echo '{}'", serialized_credentials).as_str(),
            None,
            None,
            None,
        );

        let fetched_credentials = provider.get_credentials().await.ok().unwrap();
        assert_eq!(fetched_credentials, credentials);
    }
}
