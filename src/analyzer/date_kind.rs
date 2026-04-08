use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DateKind {
    Year,
    Month,
    Day,
}

impl fmt::Display for DateKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateKind::Year => write!(f, "Year"),
            DateKind::Month => write!(f, "Month"),
            DateKind::Day => write!(f, "Day"),
        }
    }
}
