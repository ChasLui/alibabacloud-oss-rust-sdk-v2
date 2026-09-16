use std::sync::Arc;
use std::time::{Duration, SystemTime};

use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::credential::{
    new_credentials_fetcher_provider, Credentials, CredentialsFetcher, CredentialsProvider,
};
use crate::utils::option_time_rfc3339_serde;

const ECS_RAM_CRED_URL: &str = "http://100.100.100.200/latest/meta-data/ram/security-credentials/";

/// Represents the credentials obtained from an ECS role.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
/// Represents the credentials obtained from an ECS role.
pub struct EcsRoleCredentials {
    /// The access key ID associated with the ECS role.
    #[serde(rename = "AccessKeyId")]
    pub access_key_id: Option<String>,
    /// The access key secret associated with the ECS role.
    #[serde(rename = "AccessKeySecret")]
    pub access_key_secret: Option<String>,
    /// The expiration time of the ECS role credentials.
    #[serde(rename = "Expiration", with = "option_time_rfc3339_serde")]
    pub expiration: Option<SystemTime>,
    /// The security token associated with the ECS role.
    #[serde(rename = "SecurityToken")]
    pub security_token: Option<String>,
    /// The last updated time of the ECS role credentials.
    #[serde(rename = "LastUpdated", with = "option_time_rfc3339_serde")]
    pub last_updated: Option<SystemTime>,
    /// The code associated with the ECS role credentials.
    #[serde(rename = "Code")]
    pub code: Option<String>,
}

/// Represents a provider for ECS role credentials.
#[derive(Debug)]
pub struct EcsRoleCredentialsProvider {
    /// The RAM credential URL.
    pub ram_cred_url: String,
    /// The RAM role.
    pub ram_role: String,
    /// The timeout duration for the request.
    pub timeout: Duration,
    /// The number of retries for the request.
    pub retries: usize,
}

/// Options for configuring the ECS role credentials provider.
#[derive(Clone)]
pub struct EcsRoleCredentialsProviderOptions {
    /// The RAM role.
    pub ram_role: String,
    /// The timeout duration for the request.
    pub timeout: Duration,
    /// The number of retries for the request.
    pub retries: usize,
}

impl Default for EcsRoleCredentialsProviderOptions {
    fn default() -> Self {
        Self {
            ram_role: String::new(),
            timeout: Duration::from_secs(10),
            retries: 3,
        }
    }
}

#[async_trait::async_trait]
impl CredentialsProvider for EcsRoleCredentialsProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    /// Retrieves the ECS role credentials.
    ///
    /// This method is used to retrieve the credentials for an ECS role. It
    /// first checks if the `ram_role` field is empty. If it is empty, it
    /// calls the `get_role_from_metadata` method to fetch the role from the
    /// metadata. Otherwise, it uses the `ram_role` field directly. After
    /// obtaining the role, it calls the `get_credentials_from_metadata` method
    /// to get the ECS credentials. Finally, it constructs a `Credentials`
    /// object with the obtained credentials and returns it.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Credentials` object if successful, or
    /// a `Box<dyn std::error::Error >` if an error occurs.
    async fn get_credentials(
        &self,
    ) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
        let role = if self.ram_role.is_empty() {
            self.get_role_from_metadata().await? // Notice: cannot update
                                                 // self.ram_role for a
                                                 // immutable self
        } else {
            self.ram_role.clone()
        };
        let ecs_cred = self.get_credentials_from_metadata(&role).await?;

        Ok(Credentials {
            access_key_id: ecs_cred.access_key_id.unwrap_or_default(),
            access_key_secret: ecs_cred.access_key_secret.unwrap_or_default(),
            security_token: ecs_cred.security_token.unwrap_or_default(),
            expires: ecs_cred.expiration,
        })
    }
}

impl EcsRoleCredentialsProvider {
    /// Creates a new ECS role credentials provider without refreshing the
    /// credentials.
    ///
    /// # Arguments
    ///
    /// * `opts` - The options for the ECS role credentials provider.
    ///
    /// # Returns
    ///
    /// A new ECS role credentials provider.
    pub fn new_without_refresh(opts: &EcsRoleCredentialsProviderOptions) -> Self {
        Self {
            ram_cred_url: ECS_RAM_CRED_URL.to_string(),
            ram_role: opts.ram_role.clone(),
            timeout: opts.timeout,
            retries: opts.retries,
        }
    }

    /// Creates a new ECS role credentials provider.
    ///
    /// # Arguments
    ///
    /// * `_opts` - The options for the ECS role credentials provider.
    ///
    /// # Remarks
    ///
    /// This function is currently not implemented and will panic with a
    /// "waiting for CredentialsFetcherFunc" message.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_opts: &EcsRoleCredentialsProviderOptions) {
        todo!("waiting for CredentialsFetcherFunc");
    }

    /// Sends an HTTP GET request to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to send the GET request to.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `reqwest::Response` if the request is
    /// successful, or a `Box<dyn std::error::Error >` if an error occurs.
    async fn http_get(
        &self,
        url: &str,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        let client = ClientBuilder::new().timeout(self.timeout).build()?;

        let mut retry_count = 0;
        loop {
            match client.get(url).send().await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    if retry_count >= self.retries {
                        return Err(err.into());
                    }
                    retry_count += 1;
                    sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }

    /// Updates the ECS role.
    ///
    /// # Arguments
    ///
    /// * `role` - An optional new role to update to. If `None`, the role will
    ///   be retrieved from the metadata.
    ///
    /// # Returns
    ///
    /// A `Result` containing the updated role as a `String` if successful, or a
    /// `Box<dyn std::error::Error >` if an error occurs.
    #[allow(unused)]
    async fn update_role(
        &mut self,
        role: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(role) = role {
            self.ram_role = role;
        } else {
            self.ram_role = self.get_role_from_metadata().await?;
        }

        Ok(self.ram_role.clone())
    }

    /// Retrieves the ECS role from the metadata.
    ///
    /// # Returns
    ///
    /// A `Result` containing the ECS role as a `String` if successful, or a
    /// `Box<dyn std::error::Error >` if an error occurs.
    async fn get_role_from_metadata(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        //TODO retry
        let response = self.http_get(&self.ram_cred_url).await?;
        if response.status() != reqwest::StatusCode::OK {
            return Err(format!(
                "failed to fetch ecs role name, resp.StatusCode: {:?}",
                response.status()
            )
            .into());
        }
        let role_name = response.text().await?;
        if role_name.is_empty() {
            return Err("ecs role name is empty".into());
        }
        Ok(role_name)
    }

    /// Retrieves the ECS role credentials from the metadata.
    ///
    /// # Arguments
    ///
    /// * `role` - The ECS role to retrieve credentials for.
    ///
    /// # Returns
    ///
    /// A `Result` containing the ECS role credentials as an
    /// `EcsRoleCredentials` struct if successful, or a `Box<dyn
    /// std::error::Error >` if an error occurs.
    async fn get_credentials_from_metadata(
        &self,
        role: &str,
    ) -> Result<EcsRoleCredentials, Box<dyn std::error::Error + Send + Sync>> {
        let url = reqwest::Url::parse(&self.ram_cred_url)?;
        let url = url.join(role)?;

        let response = self.http_get(url.as_str()).await?;
        let body = response.text().await?;
        let ecs_cred: EcsRoleCredentials = serde_json::from_str(&body)?;

        if let Some(code) = &ecs_cred.code {
            if code.to_uppercase() != "SUCCESS" {
                return Err(format!("failed to fetch credentials, return code:{}", code).into());
            }
        }

        if ecs_cred.access_key_id.is_none() || ecs_cred.access_key_secret.is_none() {
            return Err(format!(
                "AccessKeyId or AccessKeySecret is empty, response body is '{}'",
                body
            )
            .into());
        }

        Ok(ecs_cred)
    }
}

/// Configures the RAM role for the ECS role credentials provider.
pub fn ecs_ram_role(ram_role: String) -> impl FnOnce(&mut EcsRoleCredentialsProviderOptions) {
    move |options: &mut EcsRoleCredentialsProviderOptions| {
        options.ram_role = ram_role;
    }
}

pub fn new_ecs_role_credentials_fetcher_provider(
    opts: &EcsRoleCredentialsProviderOptions,
    refresh_duration: Option<Duration>,
    expired_factor: Option<f64>,
) -> impl CredentialsProvider {
    struct EcsRoleCredentialsFetcher {
        opts: EcsRoleCredentialsProviderOptions,
    }

    impl EcsRoleCredentialsFetcher {
        pub fn new(opts: &EcsRoleCredentialsProviderOptions) -> Self {
            Self { opts: opts.clone() }
        }
    }

    #[async_trait::async_trait]
    impl CredentialsFetcher for EcsRoleCredentialsFetcher {
        async fn fetch(&self) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
            EcsRoleCredentialsProvider::new_without_refresh(&self.opts)
                .get_credentials()
                .await
        }
    }

    new_credentials_fetcher_provider(
        Arc::new(EcsRoleCredentialsFetcher::new(opts)),
        refresh_duration,
        expired_factor,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_ROUTE: &str = "/latest/meta-data/ram/security-credentials/";

    async fn mock_server(
        role: &str,
        ecs_credentials: Option<EcsRoleCredentials>,
    ) -> mockito::ServerGuard {
        let mut server = mockito::Server::new_async().await;

        let normalize_route = |route: &str| {
            let mut route = regex::Regex::new(r"/+")
                .unwrap()
                .replace_all(format!("/{}/", route).as_str(), "/+") // match "//"
                .to_string();

            // replace tailing "/+" with "/*"
            route.replace_range(route.len() - 1.., "*");

            route
        };

        server
            .mock(
                "GET",
                mockito::Matcher::Regex(format!("^{}$", normalize_route(BASE_ROUTE))),
            )
            .with_status(http::StatusCode::OK.as_u16().into())
            .with_body(role)
            .create_async()
            .await;

        if let Some(ecs_credentials) = ecs_credentials {
            server
                .mock(
                    "GET",
                    mockito::Matcher::Regex(format!(
                        "^{}$", // whole match
                        normalize_route(format!("/{}/{}/", BASE_ROUTE, role).as_str(),)
                    )),
                )
                .with_status(http::StatusCode::OK.as_u16().into())
                .with_body(serde_json::to_string(&ecs_credentials).unwrap())
                .create_async()
                .await;
        }

        server
    }

    #[tokio::test]
    async fn test_get_role() {
        let role_name = "role_name";
        let mock_server = mock_server(role_name, None).await;

        let response = EcsRoleCredentialsProvider::new_without_refresh(
            &EcsRoleCredentialsProviderOptions::default(),
        )
        .http_get(format!("{}/{}", mock_server.url(), BASE_ROUTE).as_str())
        .await
        .ok()
        .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::OK);
        assert_eq!(response.text().await.unwrap(), role_name)
    }

    #[tokio::test]
    async fn test_get_credentials() {
        let role_name = "role_name";
        let ecs_credentials = EcsRoleCredentials {
            access_key_id: Some("access_key_id".to_string()),
            access_key_secret: Some("access_key_secret".to_string()),
            security_token: Some("security_token".to_string()),
            expiration: Some(SystemTime::now() + Duration::new(3600, 0)),
            last_updated: Some(SystemTime::now()),
            code: Some("Success".to_string()),
        };
        let mock_server = mock_server(role_name, Some(ecs_credentials.clone())).await;

        let response = EcsRoleCredentialsProvider::new_without_refresh(
            &EcsRoleCredentialsProviderOptions::default(),
        )
        .http_get(format!("{}/{}/{}", mock_server.url(), BASE_ROUTE, role_name).as_str())
        .await
        .ok()
        .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::OK);

        let fetched_ecs_credentials: EcsRoleCredentials =
            serde_json::from_str(&response.text().await.unwrap()).unwrap();

        assert_eq!(
            ecs_credentials.access_key_id,
            fetched_ecs_credentials.access_key_id
        );
        assert_eq!(
            ecs_credentials.access_key_secret,
            fetched_ecs_credentials.access_key_secret
        );
        assert_eq!(
            ecs_credentials.security_token,
            fetched_ecs_credentials.security_token
        );
        assert!(
            ecs_credentials
                .expiration
                .unwrap()
                .duration_since(fetched_ecs_credentials.expiration.unwrap())
                .unwrap()
                < Duration::from_secs(1)
        );
        assert!(
            ecs_credentials
                .last_updated
                .unwrap()
                .duration_since(fetched_ecs_credentials.last_updated.unwrap())
                .unwrap()
                < Duration::from_secs(1)
        );
        assert_eq!(ecs_credentials.code, fetched_ecs_credentials.code);
    }

    #[tokio::test]
    #[ignore = "get_credentials() fetches from ECS_RAM_CRED_URL, which always fails"]
    async fn test_new_ecs_role_credentials_provider() {
        // let opts = EcsRoleCredentialsProviderOptions::default();
        // let provider = new_ecs_role_credentials_provider(&opts, None, None);
        // let credentials = provider.get_credentials().await;
    }
}
