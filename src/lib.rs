pub mod req;
pub mod resp;
pub mod setting;
mod utils;
mod workflow;
pub use utils::{
    build_tag_path, build_tag_scope, datetime_format, get_config_file, get_default_config,
};
pub use workflow::worker::deliver_image_name;

const LOGIN_URL: &str = "https://login.microsoftonline.com";

const AUTH_SCOPE: &str =
    "https://containerregistry.azure.net/.default openid offline_access profile";

const AZURE_ACR_API_VERSION: &str = "2021-07-01";

const AUTH_LOGIN_TOKEN_PATH: &str = "/oauth2/v2.0/token";
const AUTH_REFRESH_TOKEN_PATH: &str = "/oauth2/exchange";
const AUTH_FINAL_TOKEN_PATH: &str = "/oauth2/token";

pub const CATALOG_SCOPE: &str = "registry:catalog:*";
pub const CATALOG_PATH: &str = "/acr/v1/_catalog";
