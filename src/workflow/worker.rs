use crossbeam_channel::Sender;
pub async fn deliver_image_name(image_list: Vec<String>, sender: Sender<String>) {
    for image in image_list.into_iter() {
        let s_clone = sender.clone();
        tokio::spawn(async move {
            match s_clone.try_send(image) {
                Err(e) => println!("err info: {}", e),
                Ok(_) => println!("send image_name"),
            };
        });
    }
}
