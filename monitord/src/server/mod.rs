use crate::config::Config;
use crate::metrics::MetricManager;
use std::sync::{Arc, RwLock};

pub struct Server {
    pub metrics: Arc<RwLock<MetricManager>>,
}

impl Server {
    pub fn new(config: &Config) -> Self {
        let metrics = MetricManager::new();
        Server {
            metrics: Arc::new(RwLock::new(metrics)),
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Implement server logic here
        Ok(())
    }
}
