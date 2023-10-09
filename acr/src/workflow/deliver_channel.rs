use crossbeam_channel::Sender;
use requester::TagList;

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

pub async fn deliver_tag_list(tag_list: TagList, sender: Sender<TagList>) {
    if !tag_list.tags.is_empty() {
        tokio::spawn(async move {
            match sender.try_send(tag_list) {
                Err(e) => println!("err info: {}", e),
                Ok(_) => println!("send tag_list to delete"),
            };
        });
    }
}
