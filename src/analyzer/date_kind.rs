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

pub fn classify_segment(segment: &str) -> Option<DateKind> {
    if segment.len() == 4 && segment.chars().all(|c| c.is_ascii_digit()) {
        let year: u32 = segment.parse().ok()?;
        if (1900..=2100).contains(&year) {
            return Some(DateKind::Year);
        }
    }
    None
}
