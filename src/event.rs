use serde::de::Error;
use serde::{de, Deserializer};
use serde::{Deserialize, Serialize};
use std::{fmt, time::Duration};

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Record {
    Suite(suite::Event),
    Test(test::Event),
}

fn from_duration<'de, D>(d: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    struct DurationVisitor;

    impl<'de> de::Visitor<'de> for DurationVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing json data")
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Duration::from_secs_f64(v))
        }
    }

    d.deserialize_f64(DurationVisitor)
}

pub mod suite {
    use super::*;

    #[derive(Clone, Debug, Deserialize)]
    #[serde(tag = "event", rename_all = "lowercase")]
    pub enum Event {
        Started {
            test_count: u64,
        },
        Failed {
            passed: u64,
            failed: u64,
            allowed_fail: u64,
            ignored: u64,
            filtered_out: u64,
            #[serde(deserialize_with = "from_duration")]
            exec_time: Duration,
        },
    }
}

pub mod test {
    use super::*;

    #[derive(Clone, Debug, Deserialize)]
    #[serde(tag = "event", rename_all = "lowercase")]
    pub enum Event {
        Started {
            name: String,
        },
        Ok {
            name: String,
            #[serde(deserialize_with = "from_duration")]
            exec_time: Duration,
        },
        Failed {
            name: String,
            #[serde(deserialize_with = "from_duration")]
            exec_time: Duration,
            #[serde(default)]
            stdout: String,
        },
    }
}
