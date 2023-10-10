/*
    customized serialize attribute for String to charono DateTime
*/
pub mod datetime_format {
    use chrono::{DateTime, TimeZone, Timelike, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S.%fZ";
    // const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Utc.datetime_from_str(&s, FORMAT)
        //     .map_err(serde::de::Error::custom)

        // Split the input string into the main timestamp and nanoseconds parts
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() == 2 {
            let main_timestamp = parts[0];
            // complete nanosecond string by "00"
            let nanoseconds_str = format!("{}00", parts[1].trim_matches(|c| !char::is_numeric(c)));

            // Parse the main timestamp
            let mut parsed_datetime = Utc
                .datetime_from_str(main_timestamp, "%Y-%m-%dT%H:%M:%S")
                .map_err(serde::de::Error::custom)?;

            // Parse and add nanoseconds
            let nanoseconds = nanoseconds_str
                .parse::<u32>()
                .map_err(serde::de::Error::custom)?;
            parsed_datetime = parsed_datetime
                .with_nanosecond(nanoseconds)
                .ok_or_else(|| serde::de::Error::custom("Invalid nanoseconds"))?;

            return Ok(parsed_datetime);
        }

        Err(serde::de::Error::custom("Invalid timestamp format"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, TimeZone, Timelike, Utc};
    use serde::{self, Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct StructWithCustomDate {
        #[serde(with = "datetime_format")]
        timestamp: DateTime<Utc>,
        bidder: String,
    }

    #[test]
    fn test_deserialize_datetime() {
        let json_str = r#"
        {
        "timestamp": "2023-07-28T02:22:06.3574487Z",
        "bidder": "Skrillex"
        }
        "#;
        let data: StructWithCustomDate = serde_json::from_str(json_str).unwrap();
        assert_eq!(
            data,
            StructWithCustomDate {
                timestamp: Utc
                    .with_ymd_and_hms(2023, 7, 28, 2, 22, 6)
                    .unwrap()
                    .with_nanosecond(357448700)
                    .unwrap(),
                bidder: "Skrillex".to_string()
            }
        )
    }
}
