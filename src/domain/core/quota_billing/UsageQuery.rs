use super::GroupByDimension::GroupByDimension;

#[derive(Debug, Clone, Default)]
pub struct UsageQuery {
    pub tenant_id: Option<String>,
    pub app_id: Option<String>,
    pub model: Option<String>,
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    pub group_by: Vec<GroupByDimension>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
