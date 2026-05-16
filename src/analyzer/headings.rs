use dom_content_extraction::scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use crate::output::RenderOutput;

static H_SELECTORS: LazyLock<(
    Selector,
    Selector,
    Selector,
    Selector,
    Selector,
    Selector,
)> = LazyLock::new(|| {
    (
        Selector::parse("h1").unwrap(),
        Selector::parse("h2").unwrap(),
        Selector::parse("h3").unwrap(),
        Selector::parse("h4").unwrap(),
        Selector::parse("h5").unwrap(),
        Selector::parse("h6").unwrap(),
    )
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Headings {
    pub h1: Vec<String>,
    pub h2: Vec<String>,
    pub h3: Vec<String>,
    pub h4: Vec<String>,
    pub h5: Vec<String>,
    pub h6: Vec<String>,
}

impl Headings {
    pub fn is_empty(&self) -> bool {
        self.h1.is_empty()
            && self.h2.is_empty()
            && self.h3.is_empty()
            && self.h4.is_empty()
            && self.h5.is_empty()
            && self.h6.is_empty()
    }

    fn as_pairs(&self) -> [(&str, &[String]); 6] {
        [
            ("h1", &self.h1),
            ("h2", &self.h2),
            ("h3", &self.h3),
            ("h4", &self.h4),
            ("h5", &self.h5),
            ("h6", &self.h6),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct HeadingsOutput {
    pub url: String,
    pub verbosity: HeadingsVerbosity,
    pub headings: Headings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingsVerbosity {
    Main,
    Extended,
    All,
}

impl HeadingsVerbosity {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "main" => Some(Self::Main),
            "extended" => Some(Self::Extended),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Extended => "extended",
            Self::All => "all",
        }
    }
}

pub fn extract_headings(document: &Html) -> Headings {
    let (s1, s2, s3, s4, s5, s6) = &*H_SELECTORS;
    Headings {
        h1: collect_texts(document, s1),
        h2: collect_texts(document, s2),
        h3: collect_texts(document, s3),
        h4: collect_texts(document, s4),
        h5: collect_texts(document, s5),
        h6: collect_texts(document, s6),
    }
}

fn collect_texts(document: &Html, selector: &Selector) -> Vec<String> {
    document
        .select(selector)
        .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn select_headings(
    headings: &Headings,
    verbosity: HeadingsVerbosity,
) -> Headings {
    let max_level = match verbosity {
        HeadingsVerbosity::Main => 1,
        HeadingsVerbosity::Extended => 2,
        HeadingsVerbosity::All => 6,
    };
    Headings {
        h1: if max_level >= 1 {
            headings.h1.clone()
        } else {
            vec![]
        },
        h2: if max_level >= 2 {
            headings.h2.clone()
        } else {
            vec![]
        },
        h3: if max_level >= 3 {
            headings.h3.clone()
        } else {
            vec![]
        },
        h4: if max_level >= 4 {
            headings.h4.clone()
        } else {
            vec![]
        },
        h5: if max_level >= 5 {
            headings.h5.clone()
        } else {
            vec![]
        },
        h6: if max_level >= 6 {
            headings.h6.clone()
        } else {
            vec![]
        },
    }
}

impl HeadingsOutput {
    fn render_value(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("url".into(), serde_json::Value::String(self.url.clone()));
        map.insert(
            "verbosity".into(),
            serde_json::Value::String(self.verbosity.as_str().into()),
        );
        for (tag, texts) in self.headings.as_pairs() {
            map.insert(
                tag.into(),
                serde_json::Value::Array(
                    texts
                        .iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }
        serde_json::Value::Object(map)
    }
}

impl RenderOutput for HeadingsOutput {
    fn render_text(&self) -> String {
        if self.headings.is_empty() {
            return String::new();
        }

        let mut out = String::new();
        out.push_str("## Headings\n");
        out.push_str(&format!("Verbosity: {}\n", self.verbosity.as_str()));
        for (tag, texts) in self.headings.as_pairs() {
            if texts.is_empty() {
                continue;
            }
            out.push_str(&format!("\n### {tag}\n"));
            for text in texts {
                out.push_str(&format!("- {text}\n"));
            }
        }
        out
    }

    fn render_json(&self) -> String {
        serde_json::to_string_pretty(&self.render_value()).unwrap_or_default()
    }

    fn render_toon(&self) -> String {
        toon_format::encode_default(&self.render_value()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(html: &str) -> Html {
        Html::parse_document(html)
    }

    #[test]
    fn extract_all_levels() {
        let d = doc("<h1>A</h1><h2>B</h2><h3>C</h3><h4>D</h4><h5>E</h5><h6>F</h6>");
        let h = extract_headings(&d);
        assert_eq!(h.h1, vec!["A"]);
        assert_eq!(h.h2, vec!["B"]);
        assert_eq!(h.h3, vec!["C"]);
        assert_eq!(h.h4, vec!["D"]);
        assert_eq!(h.h5, vec!["E"]);
        assert_eq!(h.h6, vec!["F"]);
    }

    #[test]
    fn extract_skips_empty() {
        let d = doc("<h1>  </h1><h2></h2><h3>Ok</h3>");
        let h = extract_headings(&d);
        assert!(h.h1.is_empty());
        assert!(h.h2.is_empty());
        assert_eq!(h.h3, vec!["Ok"]);
    }

    #[test]
    fn extract_multiple_same_level() {
        let d = doc("<h1>First</h1><h1>Second</h1>");
        let h = extract_headings(&d);
        assert_eq!(h.h1, vec!["First", "Second"]);
    }

    #[test]
    fn select_main_keeps_h1_only() {
        let headings = Headings {
            h1: vec!["A".into()],
            h2: vec!["B".into()],
            h3: vec!["C".into()],
            h4: vec!["D".into()],
            h5: vec!["E".into()],
            h6: vec!["F".into()],
        };
        let filtered = select_headings(&headings, HeadingsVerbosity::Main);
        assert_eq!(filtered.h1, vec!["A"]);
        assert!(filtered.h2.is_empty());
        assert!(filtered.h3.is_empty());
    }

    #[test]
    fn select_extended_keeps_h1_h2() {
        let headings = Headings {
            h1: vec!["A".into()],
            h2: vec!["B".into()],
            h3: vec!["C".into()],
            h4: vec![],
            h5: vec![],
            h6: vec![],
        };
        let filtered = select_headings(&headings, HeadingsVerbosity::Extended);
        assert_eq!(filtered.h1, vec!["A"]);
        assert_eq!(filtered.h2, vec!["B"]);
        assert!(filtered.h3.is_empty());
    }

    #[test]
    fn select_all_keeps_everything() {
        let headings = Headings {
            h1: vec!["A".into()],
            h2: vec!["B".into()],
            h3: vec!["C".into()],
            h4: vec!["D".into()],
            h5: vec!["E".into()],
            h6: vec!["F".into()],
        };
        let filtered = select_headings(&headings, HeadingsVerbosity::All);
        assert_eq!(filtered.h1, vec!["A"]);
        assert_eq!(filtered.h6, vec!["F"]);
    }

    #[test]
    fn render_text_skips_empty_levels() {
        let output = HeadingsOutput {
            url: "https://example.com".into(),
            verbosity: HeadingsVerbosity::All,
            headings: Headings {
                h1: vec!["Title".into()],
                h2: vec![],
                h3: vec!["Sub".into()],
                h4: vec![],
                h5: vec![],
                h6: vec![],
            },
        };
        let text = output.render_text();
        assert!(text.contains("### h1"));
        assert!(text.contains("- Title"));
        assert!(text.contains("### h3"));
        assert!(!text.contains("### h2"));
    }

    #[test]
    fn render_text_empty_headings() {
        let output = HeadingsOutput {
            url: "https://example.com".into(),
            verbosity: HeadingsVerbosity::All,
            headings: Headings {
                h1: vec![],
                h2: vec![],
                h3: vec![],
                h4: vec![],
                h5: vec![],
                h6: vec![],
            },
        };
        assert!(output.render_text().is_empty());
    }
}
