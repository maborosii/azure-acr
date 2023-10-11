use crate::{
    resp::{FinalToken, LoginToken, Primary, RefreshToken, Token},
    setting::Config,
    AUTH_FINAL_TOKEN_PATH, AUTH_LOGIN_TOKEN_PATH, AUTH_REFRESH_TOKEN_PATH, AUTH_SCOPE,
    AZURE_ACR_API_VERSION, LOGIN_URL,
};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use std::{fmt::Debug, sync::Arc};

// pub type LocalResult<T> = Result<T, Box<dyn std::error::Error>>;
// pub type LocalAsyncResult<T> = Result<T, Box<dyn std::error::Error + Sync + Send>>;
pub enum GrantType {
    ClientCredentials,
    AccessToken,
    RefreshToken,
    Unknown,
}
impl From<GrantType> for &str {
    fn from(grant_type: GrantType) -> Self {
        match grant_type {
            GrantType::ClientCredentials => "client_credentials",
            GrantType::AccessToken => "access_token",
            GrantType::RefreshToken => "refresh_token",
            GrantType::Unknown => "unknown",
        }
    }
}
impl From<&str> for GrantType {
    fn from(value: &str) -> Self {
        match value {
            "client_credentials" => Self::ClientCredentials,
            "access_token" => Self::AccessToken,
            "refresh_token" => Self::RefreshToken,
            _ => Self::Unknown,
        }
    }
}

#[async_trait]
pub trait Sender {
    type Output: Token;
    async fn send(&self, config: &Config, client: Arc<reqwest::Client>) -> Result<Self::Output>;
}
#[async_trait]
impl Sender for Primary {
    type Output = LoginToken;
    async fn send(&self, config: &Config, client: Arc<reqwest::Client>) -> Result<Self::Output> {
        let login_url = format!(
            "{}/{}{}",
            LOGIN_URL,
            config.azure_tenant_id(),
            AUTH_LOGIN_TOKEN_PATH
        );
        let params = [
            ("grant_type", GrantType::ClientCredentials.into()),
            ("client_id", config.azure_acr_image_manager_id()),
            ("client_secret", config.azure_acr_image_manager_pwd()),
            ("scope", AUTH_SCOPE),
        ];

        let body = client
            .post(login_url)
            .form(&params)
            .send()
            .await?
            .json::<LoginToken>()
            .await?;

        Ok(body)
    }
}

#[async_trait]
impl Sender for LoginToken {
    type Output = RefreshToken;
    async fn send(&self, config: &Config, client: Arc<reqwest::Client>) -> Result<Self::Output> {
        let params = [
            ("grant_type", GrantType::AccessToken.into()),
            ("access_token", &self.token()[..]),
            ("service", config.azure_acr_endpoint()),
        ];

        let refresh_url = format!(
            "https://{}{}",
            config.azure_acr_endpoint(),
            AUTH_REFRESH_TOKEN_PATH
        );
        let body = client
            .post(refresh_url)
            .query(&[("api-version", AZURE_ACR_API_VERSION)])
            .form(&params)
            .send()
            .await?
            .json::<RefreshToken>()
            .await?;

        Ok(body)
    }
}

// get repos catalog access token
impl RefreshToken {
    pub async fn get_final_token(
        &self,
        config: &Config,
        client: Arc<reqwest::Client>,
        scope: &str,
    ) -> Result<FinalToken> {
        let final_token_url = format!(
            "https://{}{}",
            config.azure_acr_endpoint(),
            AUTH_FINAL_TOKEN_PATH
        );
        let params = [
            ("grant_type", GrantType::RefreshToken.into()),
            ("refresh_token", &self.token()[..]),
            ("service", config.azure_acr_endpoint()),
            ("scope", scope),
        ];

        let body = client
            .post(final_token_url)
            .query(&[("api-version", AZURE_ACR_API_VERSION)])
            .form(&params)
            .send()
            .await?
            .json::<FinalToken>()
            .await?;

        Ok(body)
    }
}

impl FinalToken {
    // get data
    pub async fn get_final_data<T>(
        &self,
        config: &Config,
        client: Arc<reqwest::Client>,
        path: &str,
    ) -> Result<T>
    where
        T: DeserializeOwned + Debug,
    {
        let catalog_url = format!("https://{}{}", config.azure_acr_endpoint(), path);
        let authorization = format!("Bearer {}", self.token());

        let body = client
            .get(catalog_url)
            .query(&[("api-version", AZURE_ACR_API_VERSION)])
            .header("Authorization", authorization)
            .send()
            .await?
            .json::<T>()
            .await?;

        Ok(body)
    }
}

impl FinalToken {
    // delete data
    pub async fn delete_image_by_tag(
        &self,
        config: &Config,
        client: Arc<reqwest::Client>,
        path: &str,
    ) -> Result<StatusCode> {
        let catalog_url = format!("https://{}{}", config.azure_acr_endpoint(), path);
        let authorization = format!("Bearer {}", self.token());

        let http_status = client
            .delete(catalog_url)
            .query(&[("api-version", AZURE_ACR_API_VERSION)])
            .header("Authorization", authorization)
            .send()
            .await?
            .status();

        Ok(http_status)
    }
}
