pub fn build_tag_scope(registry: &str) -> String {
    format!("repository:{}:metadata_read", registry)
}
pub fn build_tag_path(registry: &str) -> String {
    format!("/acr/v1/{}/_tags", registry)
}

pub fn build_delete_tag_scope(registry: &str) -> String {
    format!("repository:{}:delete", registry)
}
pub fn build_delete_tag_path(registry: &str, tag: &str) -> String {
    format!("/acr/v1/{}/_tags/{}", registry, tag)
}
