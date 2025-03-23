/// Filter options for process information requests
#[derive(Debug, Clone, Default)]
pub struct ProcessFilter {
    /// Filter processes by username
    pub username_filter: Option<String>,
    
    /// Filter processes by process ID
    pub pid_filter: Option<u32>,
    
    /// Filter processes by name (substring match)
    pub name_filter: Option<String>,
    
    /// Sort results by CPU usage (descending)
    pub sort_by_cpu: bool,
    
    /// Sort results by memory usage (descending)
    pub sort_by_memory: bool,
    
    /// Maximum number of processes to return
    pub limit: u32,
}