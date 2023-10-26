use anyhow::Result;
use requester::{Config, RefreshToken};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use std::{fmt::Debug, sync::Arc};

pub async fn get_data<T>(
    refresh_token: Arc<RefreshToken>,
    config: Arc<Config>,
    client: Arc<reqwest::Client>,
    scope: &str,
    path: &str,
) -> Result<T>
where
    T: DeserializeOwned + Debug,
{
    let body = refresh_token
        .get_final_token(&config, client.clone(), scope)
        .await?
        .get_final_data::<T>(&config, client.clone(), path)
        .await?;
    Ok(body)
}

pub async fn delete_data(
    refresh_token: Arc<RefreshToken>,
    config: Arc<Config>,
    client: Arc<reqwest::Client>,
    scope: &str,
    tag_path: &str,
    digest_path: &str,
) -> Result<StatusCode> {
    let tag_body = refresh_token
        .get_final_token(&config, client.clone(), scope)
        .await?
        .delete_image_by_tag_or_digest(&config, client.clone(), tag_path)
        .await?;
    let _ = refresh_token
        .get_final_token(&config, client.clone(), scope)
        .await?
        .delete_image_by_tag_or_digest(&config, client.clone(), digest_path)
        .await?;
    Ok(tag_body)
}
