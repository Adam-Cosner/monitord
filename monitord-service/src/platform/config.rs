#[derive(Debug, Clone)]
pub struct PlatformConfig {
    pub service_name: String,
    pub description: String,
    pub executable_path: String,
    pub user: Option<String>,
    pub group: Option<String>,
    pub working_directory: Option<String>,
    pub init_system: Option<InitSystem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InitSystem {
    SystemD,
    SysVInit,
    OpenRC,
    Runit,
    Auto,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            service_name: "monitord".to_string(),
            description: "System monitoring daemon".to_string(),
            executable_path: "/usr/bin/monitord".to_string(),
            user: None,
            group: None,
            working_directory: None,
            init_system: Some(InitSystem::Auto),
        }
    }
}
