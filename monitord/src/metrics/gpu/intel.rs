/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::Result;

pub struct IntelMetricCache {
    // Implementation details
}

impl IntelMetricCache {
    pub fn new() -> Result<Self> {
        // Implementation details
        Ok(Self {})
    }

    pub fn collect(
        &self,
        id: String,
        request: &monitord_types::service::GpuRequest,
    ) -> Result<monitord_types::service::GpuResponse> {
        // Implementation details
        Err(crate::error::Error::NotImplemented(
            "Intel GPU metrics are not implemented".to_string(),
        ))
    }
}
