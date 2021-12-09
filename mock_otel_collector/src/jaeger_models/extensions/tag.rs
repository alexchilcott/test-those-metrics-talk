use crate::jaeger_models::{Tag, TagType};
use anyhow::{anyhow, bail};

#[derive(PartialEq, Debug)]
pub enum TagValue<'t> {
    String(&'t str),
    Double(f64),
    Bool(bool),
    Long(i64),
    Binary(&'t Vec<u8>),
}

impl Tag {
    pub fn value(&self) -> Result<TagValue, anyhow::Error> {
        Ok(match self.v_type {
            TagType::BINARY => TagValue::Binary(
                self.v_binary
                    .as_ref()
                    .ok_or_else(|| anyhow!("Tag type was binary but no value was found"))?,
            ),
            TagType::BOOL => TagValue::Bool(
                self.v_bool
                    .ok_or_else(|| anyhow!("Tag type was bool but no value was found"))?,
            ),
            TagType::DOUBLE => TagValue::Double(
                self.v_double
                    .ok_or_else(|| anyhow!("Tag type was double but no value was found"))?
                    .into(),
            ),
            TagType::LONG => TagValue::Long(
                self.v_long
                    .ok_or_else(|| anyhow!("Tag type was long but no value was found"))?,
            ),
            TagType::STRING => TagValue::String(
                self.v_str
                    .as_ref()
                    .ok_or_else(|| anyhow!("Tag type was string but no value was found"))?,
            ),
            _ => bail!(format!("Unknown tag type: {}", self.v_type.0)),
        })
    }
}
