/*
    build request parameters
*/

// api: get repo list
// request params: scope
pub fn build_repos_scope() -> String {
    "registry:catalog:*".to_string()
}
// api: get repo list
// request uri path
pub fn build_repos_path() -> String {
    "/acr/v1/_catalog".to_string()
}

// api: get tag list for specific image
// request params: scope
pub fn build_tag_scope(registry: &str) -> String {
    format!("repository:{}:metadata_read", registry)
}
// api: get tag list for specific image
// request uri path
pub fn build_tag_path(registry: &str) -> String {
    format!("/acr/v1/{}/_tags", registry)
}

// api: delete tag for specific image
// request params: scope
pub fn build_delete_tag_scope(registry: &str) -> String {
    format!("repository:{}:delete", registry)
}
// api: delete tag for specific image
// request uri path
pub fn build_delete_tag_path(registry: &str, tag: &str) -> String {
    format!("/acr/v1/{}/_tags/{}", registry, tag)
}
