use std::rc::Rc;

use crossbeam_channel::Sender;
use requester::{RepositoriesList, TagList};

pub async fn deliver_image_name(image_list: RepositoriesList, sender: Sender<String>) {
    for image in image_list.repositories().into_iter() {
        let s_clone = sender.clone();
        tokio::spawn(async move {
            let image_rc = Rc::new(image);
            match s_clone.try_send(image_rc.to_string()) {
                Err(e) => println!("sender: channel[repo], msg: {{ err_info: {} }}", e),
                Ok(_) => println!("sender: channel[repo], msg: {{ image_name: {} }}", image_rc),
            };
        });
    }
}

pub async fn deliver_tag_list(tag_list: TagList, sender: Sender<TagList>) {
    if !tag_list.tags.is_empty() {
        tokio::spawn(async move {
            let tl_clone = tag_list.clone();
            match sender.try_send(tag_list) {
                Err(e) => println!("sender: channel[tags], msg: {{ err_info: {} }}", e),
                Ok(_) => println!(
                    "sender: channel[tags], msg: {{ image_name: {}, tag: {} }}",
                    &tl_clone.image_name,
                    tl_clone.tags()
                ),
            };
        });
    }
}
