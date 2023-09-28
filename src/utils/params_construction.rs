pub fn build_tag_scope(params: &str) -> String {
    format!("repository:{}:metadata_read", params)
}

pub fn build_tag_path(params: &str) -> String {
    format!("/acr/v1/{}/_tags", params)
}
