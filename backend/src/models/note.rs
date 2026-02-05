use serde::Serialize;

/// Lightweight note representation for list views.
/// Read-only, derived from filesystem + frontmatter.
#[derive(Debug, Serialize)]
pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub path: String,
    pub note_type: String,
    pub updated: Option<String>,
}

/// Full note payload for editor view.
/// Returned by GET /api/notes/:id
#[derive(Debug, Serialize)]
pub struct Note {
    pub id: String,
    pub path: String,
    pub note_type: String,
    pub frontmatter: serde_yaml::Mapping,
    pub content: String,
}
