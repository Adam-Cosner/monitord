/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

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
