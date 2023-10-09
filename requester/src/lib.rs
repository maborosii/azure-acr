mod req;
mod resp;
mod setting;
pub use req::*;
pub use resp::*;
pub use setting::*;
// pub mod setting;
// pub use utils::{
//     build_delete_tag_path, build_delete_tag_scope, build_repos_path, build_repos_scope,
//     build_tag_path, build_tag_scope, datetime_format, get_config_file, get_default_config,
// };
// pub use workflow::{deliver_image_name, deliver_tag_list};

pub const LOGIN_URL: &str = "https://login.microsoftonline.com";

pub const AUTH_SCOPE: &str =
    "https://containerregistry.azure.net/.default openid offline_access profile";
pub const AZURE_ACR_API_VERSION: &str = "2021-07-01";
pub const AUTH_LOGIN_TOKEN_PATH: &str = "/oauth2/v2.0/token";
pub const AUTH_REFRESH_TOKEN_PATH: &str = "/oauth2/exchange";
pub const AUTH_FINAL_TOKEN_PATH: &str = "/oauth2/token";
