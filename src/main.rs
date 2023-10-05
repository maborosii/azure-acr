use acr::{
    build_tag_path, build_tag_scope, deliver_image_name, get_default_config,
    req::Sender,
    resp::{self, RefreshToken, RepositoriesList, TagList},
    setting::{self, Config},
    CATALOG_PATH, CATALOG_SCOPE,
};

use anyhow::Result;
use serde::de::DeserializeOwned;
use std::{fmt::Debug, sync::Arc, thread, time::Duration};
// use tokio:`:join;

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
    let (tx, rx) = crossbeam_channel::unbounded();
    let repo_list_refresh_token = refresh_token.clone();
    let repo_list_client = client.clone();
    let repo_list_config = config.clone();
    let repo_list_task = tokio::spawn(async move {
        let image_list = get_data::<RepositoriesList>(
            repo_list_refresh_token,
            repo_list_config,
            repo_list_client,
            CATALOG_SCOPE,
            CATALOG_PATH,
        )
        .await;
        match image_list {
            anyhow::Result::Ok(images) => {
                deliver_image_name(images.filter_image_name_by_mark("/").repositories(), tx).await
            }
            Err(e) => println!("get image list err, message: {}", e),
        }
    });

    loop {
        match rx.try_recv() {
            anyhow::Result::Ok(image_name) => {
                println!("receiving image_name...{:?}", &image_name);
                let tag_list_refresh_token = refresh_token.clone();
                let tag_list_client = client.clone();
                let tag_scope = build_tag_scope(&image_name);
                let tag_path = build_tag_path(&image_name);
                let tag_list_config = config.clone();
                let _ = tokio::spawn(async move {
                    let demo = get_data::<TagList>(
                        tag_list_refresh_token.clone(),
                        tag_list_config,
                        tag_list_client,
                        &tag_scope,
                        &tag_path,
                    )
                    .await;
                    if let Err(e) = demo {
                        println!("{}", e);
                    // !TODO
                    } else {
                        let a = demo.unwrap();
                        let b = a
                            .filter_tag_by_mark("stable")
                            .filter_tag_by_mark("latest")
                            .sort_by_tag_createdtime_desc()
                            .filter_tag_by_place(3);

                        println!("{:?}", b)
                    }
                    // !
                })
                .await;
            }
            Err(crossbeam_channel::TryRecvError::Empty) => {
                println!("the channel is empty, continue listening...");
                thread::sleep(Duration::from_secs(1));
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                println!("the channel is closed, loop exitting");
                break;
            }
        }
    }

    repo_list_task.await.unwrap();
    // join!(repo_list_task);
    // get repo list
    anyhow::Ok(())
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
