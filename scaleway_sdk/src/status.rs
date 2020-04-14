use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer};

// Currently the status for namespaces and images share the same values.
#[derive(Debug, Clone)]
pub enum Status {
    Unknown,
    Ready,
    Deleting,
    Error,
    Locked,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl FromStr for Status {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Status, Self::Err> {
        match s {
            "unknown" => Ok(Status::Unknown),
            "ready" => Ok(Status::Ready),
            "deleting" => Ok(Status::Deleting),
            "error" => Ok(Status::Error),
            "locked" => Ok(Status::Locked),
            _ => Err("invalid status"),
        }
    }
}

impl AsRef<str> for Status {
    fn as_ref(&self) -> &'static str {
        match self {
            Status::Unknown => "unknown",
            Status::Ready => "ready",
            Status::Deleting => "deleting",
            Status::Error => "error",
            Status::Locked => "locked",
        }
    }
}

impl Status {
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Status, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        s.parse::<Status>().map_err(serde::de::Error::custom)
    }
}
