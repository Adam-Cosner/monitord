/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod gpu_metrics;

pub struct Card {
    // todo
}

impl Card {
    pub fn new(path: std::path::PathBuf) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

impl super::Card for Card {
    fn collect(&mut self, config: &super::Config) -> anyhow::Result<super::Gpu> {
        todo!()
    }
    fn resolve(
        &mut self,
        staging: &crate::collector::staging::Staging,
        output: super::Gpu,
    ) -> anyhow::Result<super::Gpu> {
        todo!()
    }
}
