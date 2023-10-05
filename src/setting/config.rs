use anyhow::Result;
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize)]
pub struct Config {
    azure: AzureAuth,
    acr: AcrAuth,
    filter: Option<Filter>,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let file = fs::read_to_string(path)?;
        // parse error panic
        let config: Self = toml::from_str(&file).unwrap();
        Ok(config)
    }
    pub fn azure_tenant_id(&self) -> &str {
        &self.azure.tenant_id[..]
    }
    pub fn azure_acr_image_manager_id(&self) -> &str {
        &self.acr.image_manager_id[..]
    }
    pub fn azure_acr_image_manager_pwd(&self) -> &str {
        &self.acr.image_manager_pwd[..]
    }
    pub fn azure_acr_endpoint(&self) -> &str {
        &self.acr.endpoint[..]
    }
}
#[derive(Deserialize)]
pub struct AzureAuth {
    tenant_id: String,
}

#[derive(Deserialize)]
pub struct AcrAuth {
    image_manager_id: String,
    image_manager_pwd: String,
    endpoint: String,
}

#[derive(Deserialize)]
pub struct Filter {
    image_name: ImageRule,
    tag: TagRule,
}

#[derive(Deserialize)]
pub struct ImageRule {
    keep: KeepRule,
}

#[derive(Deserialize)]
pub struct TagRule {
    keep: KeepRule,
}

#[derive(Deserialize)]
pub struct KeepRule {
    default: Option<DefaultRule>,
    rules: Option<Vec<Rule>>,
}

#[derive(Deserialize)]
pub struct DefaultRule {
    num: usize,
}

#[cfg(not(debug_assertions))]
#[derive(Deserialize)]
pub struct Rule {
    keyword: String,
}

#[cfg(debug_assertions)]
#[derive(Deserialize, PartialEq, Debug)]
pub struct Rule {
    keyword: String,
}

#[cfg(test)]
mod tests {
    use super::{Config, Rule};

    #[test]
    fn test_config_deserialize() {
        let str_keep_rule = r#"
        [azure]
        tenant_id ="tenant_id"
        [acr]
        image_manager_id = "image_manager_id"
        image_manager_pwd = "image_manager_pwd"
        endpoint = "endpoint"
        # image name filter
        [[filter.image_name.keep.rules]]
        keyword = "/"
        
        # tag filter
        [filter.tag.keep]
        default.num = 20
        [[filter.tag.keep.rules]]
        keyword = "stable"
        [[filter.tag.keep.rules]]
        keyword = "latest"

        "#;
        let config: Config = toml::from_str(str_keep_rule).unwrap();
        let filter_rule = &config.filter.unwrap();
        let image_keep_rule = &filter_rule.image_name.keep;
        let tag_keep_rule = &filter_rule.tag.keep;
        assert_eq!(config.azure.tenant_id, "tenant_id");
        assert_eq!(config.acr.image_manager_id, "image_manager_id");
        assert_eq!(config.acr.image_manager_pwd, "image_manager_pwd");
        assert_eq!(config.acr.endpoint, "endpoint");

        let tag_keep_rule_default = tag_keep_rule.default.as_ref().unwrap();

        assert_eq!(tag_keep_rule_default.num, 20);
        assert_eq!(
            tag_keep_rule.rules,
            Some(vec![
                Rule {
                    keyword: "stable".to_string()
                },
                Rule {
                    keyword: "latest".to_string()
                }
            ])
        );
        assert_eq!(
            image_keep_rule.rules,
            Some(vec![Rule {
                keyword: "/".to_string()
            }])
        );
    }
}
