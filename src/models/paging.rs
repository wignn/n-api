use serde::{Deserialize, Serialize};



#[derive(Debug, Deserialize, Clone)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    10
}

impl PaginationParams {
    pub fn skip(&self) -> i64 {
        (self.page - 1) * self.limit
    }


    pub fn take(&self) -> i64 {
        self.limit
    }


    pub fn validate(&self) -> Result<(), String> {
        if self.page < 1 {
            return Err("Page must be greater than 0".to_string());
        }
        if self.limit < 1 || self.limit > 100 {
            return Err("Limit must be between 1 and 100".to_string());
        }
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub current_page: i64,
    pub per_page: i64,
    pub total_items: i64,
    pub total_pages: i64,
    pub has_previous: bool,
    pub has_next: bool,
}

impl PaginationMeta {
    pub fn new(current_page: i64, per_page: i64, total_items: i64) -> Self {
        let total_pages = if per_page > 0 {
            (total_items as f64 / per_page as f64).ceil() as i64
        } else {
            0
        };

        Self {
            current_page,
            per_page,
            total_items,
            total_pages,
            has_previous: current_page > 1,
            has_next: current_page < total_pages,
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, pagination: PaginationMeta) -> Self {
        Self { data, pagination }
    }
}

