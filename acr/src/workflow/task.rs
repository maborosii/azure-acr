use super::{deliver_image_name, deliver_tag_list, get_data};
use requester::{Config, RefreshToken, RepositoriesList, TagList};
use reqwest::Client;
use std::{sync::Arc, thread, time::Duration};
use utils::{build_delete_tag_path, build_delete_tag_scope, build_tag_path, build_tag_scope};

pub async fn create_repo_list_task(
    repo_list_refresh_token: Arc<RefreshToken>,
    repo_list_config: Arc<Config>,
    repo_list_client: Arc<Client>,
    repo_scope: &str,
    repo_path: &str,
    repo_tx: crossbeam_channel::Sender<String>,
) {
    let image_list = get_data::<RepositoriesList>(
        repo_list_refresh_token,
        repo_list_config,
        repo_list_client,
        repo_scope,
        repo_path,
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
}
