use crate::config::Config;
use std::sync::{Arc, RwLock};

pub struct Server {}

impl Server {
    pub fn new(config: &Config) -> Self {
        Server {}
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Implement server logic here
        Ok(())
    }
}
