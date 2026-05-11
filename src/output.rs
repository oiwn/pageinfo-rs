#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Toon,
}

impl OutputFormat {
    /// Parses a CLI output format value.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "text" => Some(Self::Text),
            "json" => Some(Self::Json),
            "toon" => Some(Self::Toon),
            _ => None,
        }
    }
}

pub trait RenderOutput {
    /// Renders output for human-readable terminal use.
    fn render_text(&self) -> String;

    /// Renders output as structured JSON.
    fn render_json(&self) -> String;

    /// Renders output as compact TOON for LLM context.
    fn render_toon(&self) -> String;

    /// Renders output using the requested output format.
    fn render(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Text => self.render_text(),
            OutputFormat::Json => self.render_json(),
            OutputFormat::Toon => self.render_toon(),
        }
    }
}
