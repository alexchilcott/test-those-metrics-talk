use crate::jaeger_models::{Span, Tag};

impl Span {
    pub fn get_tag(&self, key: &str) -> Option<&Tag> {
        let tag = self.tags.as_ref()?.iter().find(|t| t.key == key);
        tag
    }

    pub fn hex_trace_id(&self) -> String {
        format!("{:016x}{:016x}", &self.trace_id_high, &self.trace_id_low)
    }
}
