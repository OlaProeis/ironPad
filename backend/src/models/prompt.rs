use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_yaml::{Mapping, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptKind {
    #[default]
    Prompt,
    Link,
}

/// Library pointer fields in prompt frontmatter (`kind`, `source`, `template`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PromptPointerFields {
    pub kind: PromptKind,
    pub source: Option<String>,
    pub template: bool,
}

impl PromptPointerFields {
    pub fn from_mapping(fm: &Mapping) -> Self {
        let kind = fm
            .get(Value::from("kind"))
            .and_then(|v| v.as_str())
            .map(|s| match s.to_lowercase().as_str() {
                "link" => PromptKind::Link,
                _ => PromptKind::Prompt,
            })
            .unwrap_or(PromptKind::Prompt);

        let source = fm
            .get(Value::from("source"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        let template = fm
            .get(Value::from("template"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Self {
            kind,
            source,
            template,
        }
    }

    /// Write pointer fields into a frontmatter mapping.
    /// Prompts omit `kind`, `source`, and `template`; links emit them when set.
    pub fn apply_to_mapping(&self, fm: &mut Mapping) {
        fm.remove(Value::from("kind"));
        fm.remove(Value::from("source"));
        fm.remove(Value::from("template"));

        if self.kind == PromptKind::Link {
            fm.insert(Value::from("kind"), Value::from("link"));
            if let Some(ref source) = self.source {
                fm.insert(Value::from("source"), Value::from(source.as_str()));
            }
            if self.template {
                fm.insert(Value::from("template"), Value::from(true));
            }
        }
    }

    pub fn prepare_mapping_for_write(fm: &mut Mapping) {
        Self::from_mapping(fm).apply_to_mapping(fm);
    }
}

impl<'de> Deserialize<'de> for PromptPointerFields {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "lowercase")]
        struct Raw {
            #[serde(default)]
            kind: PromptKind,
            source: Option<String>,
            #[serde(default)]
            template: bool,
        }

        let raw = Raw::deserialize(deserializer)?;
        Ok(Self {
            kind: raw.kind,
            source: raw.source,
            template: raw.template,
        })
    }
}

impl Serialize for PromptPointerFields {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.kind == PromptKind::Prompt {
            let map = serializer.serialize_map(Some(0))?;
            return map.end();
        }

        let mut entries = 2usize;
        if self.template {
            entries += 1;
        }
        let mut map = serializer.serialize_map(Some(entries))?;
        map.serialize_entry("kind", "link")?;
        if let Some(ref source) = self.source {
            map.serialize_entry("source", source)?;
        }
        if self.template {
            map.serialize_entry("template", &true)?;
        }
        map.end()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptSummary {
    pub id: String,
    pub title: String,
    pub path: String,
    pub scope: String,
    pub project_id: Option<String>,
    pub folder: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub updated: Option<String>,
    pub kind: PromptKind,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Prompt {
    pub id: String,
    pub title: String,
    pub path: String,
    pub scope: String,
    pub project_id: Option<String>,
    pub folder: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub kind: PromptKind,
    pub source: Option<String>,
    pub template: bool,
    pub frontmatter: serde_yaml::Mapping,
    pub content: String,
}

/// GET `/api/prompts/{id}` body. Link `content` is the source file read at request time.
#[derive(Debug, Clone, Serialize)]
pub struct PromptGetResponse {
    #[serde(flatten)]
    pub prompt: Prompt,
    pub read_only: bool,
    pub source_missing: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_rendered: Option<bool>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub root_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptFolder {
    pub path: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptSearchResult {
    pub prompt: PromptSummary,
    pub score: f32,
    pub matched_terms: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_no_kind_defaults_to_prompt() {
        let fields: PromptPointerFields = serde_yaml::from_str("title: x\n").unwrap();
        assert_eq!(fields.kind, PromptKind::Prompt);
        assert!(fields.source.is_none());
        assert!(!fields.template);
    }

    #[test]
    fn pointer_kind_prompt_explicit() {
        let fields: PromptPointerFields = serde_yaml::from_str("kind: prompt\n").unwrap();
        assert_eq!(fields.kind, PromptKind::Prompt);
    }

    #[test]
    fn pointer_link_round_trip() {
        let original = PromptPointerFields {
            kind: PromptKind::Link,
            source: Some("ai-context.md".to_string()),
            template: true,
        };
        let yaml = serde_yaml::to_string(&original).unwrap();
        let parsed: PromptPointerFields = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn pointer_serialize_prompt_omits_keys() {
        let fields = PromptPointerFields::default();
        let yaml = serde_yaml::to_string(&fields).unwrap();
        assert!(!yaml.contains("kind:"));
        assert!(!yaml.contains("source:"));
        assert!(!yaml.contains("template:"));
        let parsed: Value = serde_yaml::from_str(&yaml).unwrap();
        match parsed {
            Value::Null => {}
            Value::Mapping(m) => assert!(m.is_empty()),
            other => panic!("unexpected yaml value: {:?}", other),
        }
    }

    #[test]
    fn pointer_serialize_link_emits_kind_and_source() {
        let fields = PromptPointerFields {
            kind: PromptKind::Link,
            source: Some("ai-context.md".to_string()),
            template: false,
        };
        let yaml = serde_yaml::to_string(&fields).unwrap();
        assert!(yaml.contains("kind: link"));
        assert!(yaml.contains("source: ai-context.md"));
        assert!(!yaml.contains("template:"));
    }

    #[test]
    fn pointer_from_mapping_matches_yaml_parser() {
        let yaml = "kind: link\nsource: foo.md\ntemplate: true\n";
        let fm: Mapping = serde_yaml::from_str(yaml).unwrap();
        let from_map = PromptPointerFields::from_mapping(&fm);
        let from_yaml: PromptPointerFields = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(from_map, from_yaml);
    }

    #[test]
    fn prepare_mapping_strips_prompt_link_keys() {
        let mut fm = Mapping::new();
        fm.insert(Value::from("kind"), Value::from("prompt"));
        fm.insert(Value::from("source"), Value::from("x"));
        fm.insert(Value::from("title"), Value::from("T"));
        PromptPointerFields::prepare_mapping_for_write(&mut fm);
        assert!(!fm.contains_key(Value::from("kind")));
        assert!(!fm.contains_key(Value::from("source")));
        assert!(!fm.contains_key(Value::from("template")));
        assert_eq!(
            fm.get(&Value::from("title")).unwrap().as_str().unwrap(),
            "T"
        );
    }
}
