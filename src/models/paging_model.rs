use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    pub search: Option<String>,
    pub genres: Option<String>,
    pub sort: Option<String>, // newest, oldest, popular, alphabetical
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: i64,
    pub page_size: i64,
    pub total_items: i64,
    pub total_pages: i64,
}
