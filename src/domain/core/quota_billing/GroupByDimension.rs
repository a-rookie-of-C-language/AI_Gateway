#[derive(Debug, Clone)]
pub enum GroupByDimension {
    Tenant,
    Model,
    Hour,
    Day,
}
