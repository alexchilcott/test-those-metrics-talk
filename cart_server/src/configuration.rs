#[derive(Clone)]
pub struct Configuration {
    pub host: String,
    pub port: u16,
    pub stock_service_url: String,
    pub collector_url: String,
}
