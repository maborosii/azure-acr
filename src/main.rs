use acr::{
    build_delete_tag_path, build_delete_tag_scope, build_repos_path, build_repos_scope,
    build_tag_path, build_tag_scope, deliver_image_name, deliver_tag_list, get_default_config,
    req::Sender,
    resp::{self, RefreshToken, RepositoriesList, TagList},
    setting::{self, Config},
};

use anyhow::Result;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use std::{fmt::Debug, sync::Arc, thread, time::Duration};
use tokio::join;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Arc::new(reqwest::Client::new());
    let config = Arc::new(config().unwrap());
    let start_point = resp::Primary;

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
        let image_list = get_data::<RepositoriesList>(
            repo_list_refresh_token,
            repo_list_config,
            repo_list_client,
            &repo_scope,
            &repo_path,
        )
        .await;
        match image_list {
            Ok(images) => {
                deliver_image_name(
                    images.filter_image_name_by_mark("/").repositories(),
                    repo_tx,
                )
                .await
            }
            Err(e) => println!("get image list err, message: {}", e),
        }
    });

    let tag_list_refresh_token = refresh_token.clone();
    let tag_list_client = client.clone();
    let tag_list_config = config.clone();
    let tag_list_task = tokio::spawn(async move {
        loop {
            match repo_rx.try_recv() {
                Ok(image_name) => {
                    println!("receiving image_name...{:?}", &image_name);
                    let tag_list_refresh_token = tag_list_refresh_token.clone();
                    let tag_list_client = tag_list_client.clone();
                    let tag_scope = build_tag_scope(&image_name);
                    let tag_path = build_tag_path(&image_name);
                    let tag_list_config = tag_list_config.clone();
                    let tag_tx_clone = tag_tx.clone();
                    let _ = tokio::spawn(async move {
                        let tag_filter_config = tag_list_config.clone();
                        let tmp_tag_list = get_data::<TagList>(
                            tag_list_refresh_token.clone(),
                            tag_list_config,
                            tag_list_client,
                            &tag_scope,
                            &tag_path,
                        )
                        .await;
                        if let Err(e) = tmp_tag_list {
                            println!("{}", e);
                        } else {
                            let a = tmp_tag_list.unwrap();
                            let b = a.filter_by_tag_rule(tag_filter_config);
                            if let Ok(data) = b {
                                // println!("{:?}", data);
                                deliver_tag_list(data, tag_tx_clone).await;
                            }
                        }
                    })
                    .await;
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    println!("the channel of receiving image_name is empty, continue listening...");
                    thread::sleep(Duration::from_secs(1));
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    println!("the channel of receiving image_name is closed, loop exiting");
                    break;
                }
            }
        }
    });
    let delete_tag_list_refresh_token = refresh_token.clone();
    let delete_tag_list_client = client.clone();
    let delete_tag_list_config = config.clone();
    let delete_tag_list_task = tokio::spawn(async move {
        loop {
            match tag_rx.try_recv() {
                Ok(tag_list) => {
                    println!("receiving delete tag_list...{:?}", &tag_list);
                    let delete_tag_list_refresh_token = delete_tag_list_refresh_token.clone();
                    let delete_tag_list_client = delete_tag_list_client.clone();

                    // !TODO: loop for tags
                    let delete_tag_scope =
                        build_delete_tag_path(&tag_list.registry, &tag_list.tags[0].name);
                    let delete_tag_path = build_delete_tag_scope(&tag_list.registry);
                    // !

                    let delete_tag_list_config = delete_tag_list_config.clone();

                    // let _ = tokio::spawn(async move {
                    //     let tmp_delete_tag_list = delete_data(
                    //         delete_tag_list_refresh_token,
                    //         delete_tag_list_config,
                    //         delete_tag_list_client,
                    //         &delete_tag_scope,
                    //         &delete_tag_path,
                    //     )
                    //     .await;
                    //     if let Err(e) = tmp_delete_tag_list {
                    //         println!("{}", e);
                    //     } else if let Ok(data) = tmp_delete_tag_list {
                    //         println!("{:?}", data);
                    //     }
                    // })
                    // .await;
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    println!("the channel of receiving delete tag is empty, continue existing...");
                    thread::sleep(Duration::from_secs(1));
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    println!("the channel of receiving delete tag is closed, loop existing");
                    break;
                }
            }
        }
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

fn config() -> Result<setting::Config> {
    let config_file = get_default_config("config.toml").unwrap();
    setting::Config::load(config_file)
}

async fn get_data<T>(
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

async fn delete_data(
    refresh_token: Arc<RefreshToken>,
    config: Arc<Config>,
    client: Arc<reqwest::Client>,
    scope: &str,
    path: &str,
) -> Result<StatusCode> {
    let body = refresh_token
        .get_final_token(&config, client.clone(), scope)
        .await?
        .delete_image_by_tag(&config, client.clone(), path)
        .await?;
    Ok(body)
}
