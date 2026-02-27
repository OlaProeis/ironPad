use serde::Serialize;

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
    pub frontmatter: serde_yaml::Mapping,
    pub content: String,
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
