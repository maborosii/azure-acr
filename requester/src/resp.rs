// use crate::{datetime_format, setting::Config};
use crate::setting::Config;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::{collections::HashSet, sync::Arc};
use utils::datetime_format;

pub trait Token {
    fn token(&self) -> String;
}
// start place for chain caller
pub struct Primary;

impl Token for Primary {
    fn token(&self) -> String {
        "".to_string()
    }
}

// the first resp for login azure
#[derive(Deserialize, Debug)]
pub struct LoginToken {
    // token_type: String,
    // expires_in: i32,
    // ext_expires_in: i32,
    access_token: String,
}
impl Token for LoginToken {
    fn token(&self) -> String {
        self.access_token.to_string()
    }
}

// the second resp for get refresh token
#[derive(Deserialize, Debug)]
pub struct RefreshToken {
    refresh_token: String,
}

impl Token for RefreshToken {
    fn token(&self) -> String {
        self.refresh_token.to_string()
    }
}

// the third resp for last req（real req）
#[derive(Deserialize, Debug)]
pub struct FinalToken {
    access_token: String,
}

impl Token for FinalToken {
    fn token(&self) -> String {
        self.access_token.to_string()
    }
}

// repo list
#[derive(Deserialize, Debug)]
pub struct RepositoriesList {
    repositories: Vec<String>,
}
impl RepositoriesList {
    pub fn repositories(self) -> Vec<String> {
        self.repositories
    }
    // drop image name which contains `mark`
    pub fn filter_image_name_by_mark(mut self, mark: &str) -> Self {
        let filter_list: Vec<_> = self
            .repositories
            .into_iter()
            .filter(|x| !x.clone().contains(mark))
            .collect();
        self.repositories = filter_list;
        self
    }
    pub fn filter_by_image_rule(mut self, config: Arc<Config>) -> Result<Self> {
        match &config.filter {
            None => Err(anyhow::anyhow!("config filter rules is none")),
            Some(filter) => {
                let keep_rule = &filter.image_name.keep.rules;
                match keep_rule {
                    // if none: return all
                    None => Ok(self),
                    Some(rules) => {
                        for i in rules {
                            self = self.filter_image_name_by_mark(i.keyword.as_str());
                        }
                        Ok(self)
                    }
                }
            }
        }
    }
}

// tag list
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TagList {
    pub registry: String,
    #[serde(rename(deserialize = "imageName"))]
    pub image_name: String,
    pub tags: Vec<Tag>,
}

impl TagList {
    pub fn tags(&self) -> String {
        self.tags
            .iter()
            .map(|x| x.clone().name)
            .collect::<Vec<String>>()
            .join(",")
    }
    // sort order by tag's created time desc
    pub fn sort_by_tag_createdtime_desc(mut self) -> Self {
        self.tags
            .sort_by(|a, b| b.created_time.cmp(&a.created_time));
        self
    }
    // drop tags name which contains `mark` by its digest
    pub fn filter_tag_by_mark(mut self, mark: &str) -> Self {
        // get digest list
        let manifests_list: HashSet<_> = self
            .tags
            .iter()
            .filter(|&x| x.clone().name.contains(mark))
            .map(|x| x.clone().digest)
            .collect();
        self.tags.retain(|x| !manifests_list.contains(&x.digest));
        self
    }
    // filter tags which is in tag list's [0..`hold`]
    pub fn filter_tag_by_place(mut self, hold: usize) -> Self {
        if self.tags.len() > hold {
            self.tags = self.tags[hold..].to_vec();
        } else {
            self.tags = vec![];
        }
        self
    }
    // aggregate filter rules by config
    pub fn filter_by_tag_rule(mut self, config: Arc<Config>) -> Result<Self> {
        match &config.filter {
            None => Err(anyhow::anyhow!("config filter rules is none")),
            Some(filter) => {
                let keep_default = &filter.tag.keep.default;
                let keep_rule = &filter.tag.keep.rules;
                match (keep_default, keep_rule) {
                    // if none: do nothing
                    (None, None) => Err(anyhow::anyhow!("tag filter rules is none")),
                    (None, Some(rules)) => {
                        for i in rules {
                            self = self.filter_tag_by_mark(i.keyword.as_str());
                        }
                        Ok(self)
                    }
                    (Some(hold), None) => {
                        self = self
                            .sort_by_tag_createdtime_desc()
                            .filter_tag_by_place(hold.num);
                        Ok(self)
                    }
                    (Some(hold), Some(rules)) => {
                        for i in rules {
                            self = self.filter_tag_by_mark(i.keyword.as_str());
                        }
                        self = self
                            .sort_by_tag_createdtime_desc()
                            .filter_tag_by_place(hold.num);
                        Ok(self)
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct Tag {
    pub name: String,
    pub digest: String,

    #[serde(rename(deserialize = "createdTime"), with = "datetime_format")]
    pub created_time: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Timelike};

    use super::*;

    #[test]
    fn test_deserialize_tag_list() {
        let json_data_valid = r#"
            {
                "registry": "example_registry",
                "imageName": "example_image",
                "tags": [
                    {
                        "name": "tag1",
                        "digest": "digest1",
                        "createdTime": "2023-08-23T06:08:46.7423121Z"
                    },
                    {
                        "name": "tag2",
                        "digest": "digest2",
                        "createdTime": "2023-08-24T18:18:01.1123121Z"
                    }
                ]
            }
        "#;
        let expected_tag_list = TagList {
            registry: "example_registry".to_string(),
            image_name: "example_image".to_string(),
            tags: vec![
                Tag {
                    name: "tag1".to_string(),
                    digest: "digest1".to_string(),
                    created_time: Utc
                        .with_ymd_and_hms(2023, 8, 23, 6, 8, 46)
                        .unwrap()
                        .with_nanosecond(742312100)
                        .unwrap(),
                },
                Tag {
                    name: "tag2".to_string(),
                    digest: "digest2".to_string(),
                    created_time: Utc
                        .with_ymd_and_hms(2023, 8, 24, 18, 18, 1)
                        .unwrap()
                        .with_nanosecond(112312100)
                        .unwrap(),
                },
            ],
        };

        let tag_list: TagList = serde_json::from_str(json_data_valid).unwrap();
        assert_eq!(tag_list, expected_tag_list);
    }
}
