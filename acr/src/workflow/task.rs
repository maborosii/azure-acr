use super::{delete_data, deliver_image_name, deliver_tag_list, get_data};
use anyhow::Result;
use requester::{Config, Primary, RefreshToken, RepositoriesList, Sender, TagList};
use reqwest::Client;
use std::{sync::Arc, thread, time::Duration};
use utils::{
    build_delete_digest_path, build_delete_tag_path, build_delete_tag_scope, build_tag_path,
    build_tag_scope,
};

pub async fn create_refresh_token_task(
    config: &Config,
    client: Arc<Client>,
) -> Result<RefreshToken> {
    let start_point = Primary;
    // get refresh token
    start_point
        .send(config, client.clone())
        .await?
        .send(config, client.clone())
        .await
}

pub async fn create_repo_list_task(
    repo_list_refresh_token: Arc<RefreshToken>,
    repo_list_config: Arc<Config>,
    repo_list_client: Arc<Client>,
    repo_scope: &str,
    repo_path: &str,
    repo_tx: crossbeam_channel::Sender<String>,
) {
    let repo_filter_config = repo_list_config.clone();
    let tmp_repo_list = get_data::<RepositoriesList>(
        repo_list_refresh_token,
        repo_list_config,
        repo_list_client,
        repo_scope,
        repo_path,
    )
    .await;
    match tmp_repo_list {
        Err(e) => println!("get repo list err, msg: {{ err_info: {} }}", e),
        Ok(repos) => {
            if let Ok(data) = repos.filter_by_image_rule(repo_filter_config) {
                deliver_image_name(data, repo_tx).await
            }
        }
    }
}

pub async fn create_tag_list_task(
    tag_list_refresh_token: Arc<RefreshToken>,
    tag_list_config: Arc<Config>,
    tag_list_client: Arc<Client>,
    repo_rx: crossbeam_channel::Receiver<String>,
    tag_tx: crossbeam_channel::Sender<TagList>,
) {
    loop {
        match repo_rx.try_recv() {
            Ok(image_name) => {
                println!(
                    "receiver: channel[repo], msg: {{ image_name: {} }}",
                    &image_name
                );
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
                    match tmp_tag_list {
                        Err(e) => println!("get tag list err, msg: {{ err_info: {} }}", e),
                        Ok(tl) => {
                            if let Ok(data) = tl.filter_by_tag_rule(tag_filter_config) {
                                deliver_tag_list(data, tag_tx_clone).await;
                            }
                        }
                    }
                })
                .await;
            }
            Err(crossbeam_channel::TryRecvError::Empty) => {
                println!(
                    "receiver: channel[repo], msg: {{ err_info: the channel is empty, continue listening... }}"
                );
                thread::sleep(Duration::from_secs(1));
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                println!("receiver: channel[repo], msg: {{ err_info: is closed, loop exiting. }}");
                break;
            }
        }
    }
}

pub async fn create_delete_tag_list_task(
    delete_tag_list_refresh_token: Arc<RefreshToken>,
    delete_tag_list_config: Arc<Config>,
    delete_tag_list_client: Arc<Client>,
    tag_rx: crossbeam_channel::Receiver<TagList>,
) {
    loop {
        match tag_rx.try_recv() {
            Ok(tag_list) => {
                for tag in tag_list.tags.into_iter() {
                    let image_name = tag_list.image_name.clone();
                    let delete_tag_list_config = delete_tag_list_config.clone();
                    let delete_tag_list_refresh_token = delete_tag_list_refresh_token.clone();
                    let delete_tag_list_client = delete_tag_list_client.clone();
                    let delete_tag_scope = build_delete_tag_scope(&tag_list.image_name);
                    let delete_tag_path = build_delete_tag_path(&tag_list.image_name, &tag.name);
                    let delete_digest_path =
                        build_delete_digest_path(&tag_list.image_name, &tag.digest);

                    let _ = tokio::spawn(async move {
                        println!(
                            "receiver: channel[tags], msg: {{ image_name: {}, tag: {} }}",
                            &image_name, &tag.name
                        );
                        // delete image by tag
                        let delete_tag_result = delete_data(
                            delete_tag_list_refresh_token,
                            delete_tag_list_config,
                            delete_tag_list_client,
                            &delete_tag_scope,
                            &delete_tag_path,
                            &delete_digest_path,
                        )
                        .await;
                        match delete_tag_result {
                            Err(e) => println!("delete tag err, msg: {{ err_info: {} }}", e),
                            Ok(_) => {
                                println!(
                                    "delete tag success, msg: {{ image_name: {}, tag: {} }}",
                                    &image_name, tag.name
                                )
                            }
                        }
                    })
                    .await;
                }
            }
            Err(crossbeam_channel::TryRecvError::Empty) => {
                println!(
                    "receiver: channel[tags], msg: {{ err_info: the channel is empty, continue listening... }}"
                );
                thread::sleep(Duration::from_secs(1));
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                println!("receiver: channel[tags], msg: {{ err_info: is closed, loop exiting. }}");
                break;
            }
        }
    }
}
