use acr::workflow::{create_delete_tag_list_task, create_repo_list_task, create_tag_list_task};
use anyhow::Result;
use requester::{config, Primary, Sender};
use std::sync::Arc;
use tokio::join;
use utils::{build_repos_path, build_repos_scope};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Arc::new(reqwest::Client::new());
    let config = Arc::new(config().unwrap());
    let start_point = Primary;

    // get refresh token
    let refresh_token = Arc::new(
        start_point
            .send(&config, client.clone())
            .await?
            .send(&config, client.clone())
            .await?,
    );

    let (repo_tx, repo_rx) = crossbeam_channel::unbounded();
    let (tag_tx, tag_rx) = crossbeam_channel::unbounded();

    let repo_list_refresh_token = refresh_token.clone();
    let repo_list_client = client.clone();
    let repo_list_config = config.clone();
    let repo_scope = build_repos_scope();
    let repo_path = build_repos_path();
    let repo_list_task = tokio::spawn(async move {
        create_repo_list_task(
            repo_list_refresh_token,
            repo_list_config,
            repo_list_client,
            &repo_scope,
            &repo_path,
            repo_tx,
        )
        .await;
    });

    let tag_list_refresh_token = refresh_token.clone();
    let tag_list_client = client.clone();
    let tag_list_config = config.clone();
    let tag_list_task = tokio::spawn(async move {
        create_tag_list_task(
            tag_list_refresh_token,
            tag_list_config,
            tag_list_client,
            repo_rx,
            tag_tx,
        )
        .await;
    });

    let delete_tag_list_refresh_token = refresh_token.clone();
    let delete_tag_list_client = client.clone();
    let delete_tag_list_config = config.clone();
    let delete_tag_list_task = tokio::spawn(async move {
        create_delete_tag_list_task(
            delete_tag_list_refresh_token,
            delete_tag_list_config,
            delete_tag_list_client,
            tag_rx,
        )
        .await;
    });

    // repo_list_task.await.unwrap();
    let (repo_list_result, tag_list_result, delete_list_result) =
        join!(repo_list_task, tag_list_task, delete_tag_list_task);
    match (repo_list_result, tag_list_result, delete_list_result) {
        (Ok(_), Ok(_), Ok(_)) => Ok(()),
        (Err(repo_err), _, _) => Err(anyhow::anyhow!("get repo list err: {}", repo_err)),
        (_, Err(tag_err), _) => Err(anyhow::anyhow!("get tag list err: {}", tag_err)),
        (_, _, Err(delete_tag_err)) => {
            Err(anyhow::anyhow!("delete tag list err: {}", delete_tag_err))
        }
    }
}
