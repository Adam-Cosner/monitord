use crate::error::PlatformError;

use super::config::PlatformConfig;

pub fn register_service(_: PlatformConfig) -> Result<(), PlatformError> {
    // Todo: Register with init system
    Ok(())
}
