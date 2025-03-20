use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::error::PlatformError;
use super::config::{PlatformConfig, InitSystem};

pub fn register_service(config: PlatformConfig) -> Result<(), PlatformError> {
    // Detect init system if set to Auto
    let init_system = match &config.init_system {
        Some(InitSystem::Auto) => detect_init_system()?,
        Some(ref system) => system.clone(),
        None => InitSystem::SystemD, // Default to systemd if not specified
    };

    // Check if we need to create user and group
    if let Some(ref user) = config.user {
        ensure_user_exists(user, config.group.as_deref())?;
    }

    // Create working directory if needed
    if let Some(ref working_dir) = config.working_directory {
        ensure_directory_exists(working_dir, config.user.as_deref(), config.group.as_deref())?;
    }

    match init_system {
        InitSystem::SystemD => register_systemd_service(&config),
        InitSystem::SysVInit => register_sysvinit_service(&config),
        InitSystem::OpenRC => register_openrc_service(&config),
        InitSystem::Runit => register_runit_service(&config),
        InitSystem::Auto => unreachable!(), // This case is handled during detection
    }
}

fn detect_init_system() -> Result<InitSystem, PlatformError> {
    // Check for systemd
    if Path::new("/run/systemd/system").exists() {
        return Ok(InitSystem::SystemD);
    }

    // Check for OpenRC
    if Path::new("/etc/init.d").exists() && Command::new("rc-status").output().is_ok() {
        return Ok(InitSystem::OpenRC);
    }

    // Check for Runit
    if Path::new("/etc/runit").exists() || Path::new("/etc/sv").exists() {
        return Ok(InitSystem::Runit);
    }

    // Check for SysVInit
    if Path::new("/etc/init.d").exists() {
        return Ok(InitSystem::SysVInit);
    }

    Err(PlatformError::InitSystemDetectionFailed)
}

fn register_systemd_service(config: &PlatformConfig) -> Result<(), PlatformError> {
    let service_path = PathBuf::from("/etc/systemd/system")
        .join(format!("{}.service", config.service_name));

    // Create systemd service file content
    let mut service_content = String::new();
    service_content.push_str("[Unit]\n");
    service_content.push_str(&format!("Description={}\n", config.description));
    service_content.push_str("After=network.target\n\n");

    service_content.push_str("[Service]\n");
    service_content.push_str(&format!("ExecStart={}\n", config.executable_path));

    if let Some(user) = &config.user {
        service_content.push_str(&format!("User={}\n", user));
    }

    if let Some(group) = &config.group {
        service_content.push_str(&format!("Group={}\n", group));
    }

    if let Some(working_dir) = &config.working_directory {
        service_content.push_str(&format!("WorkingDirectory={}\n", working_dir));
    }

    service_content.push_str("Restart=on-failure\n\n");

    service_content.push_str("[Install]\n");
    service_content.push_str("WantedBy=multi-user.target\n");

    // Write file
    fs::write(&service_path, service_content)
        .map_err(|e| PlatformError::Io(e))?;

    // Reload systemd daemon
    Command::new("systemctl")
        .args(["daemon-reload"])
        .output()
        .map_err(|e| PlatformError::Io(e))?;

    println!("Registered systemd service at: {}", service_path.display());
    println!("To enable and start the service, run:");
    println!("sudo systemctl enable --now {}", config.service_name);

    Ok(())
}

fn register_sysvinit_service(config: &PlatformConfig) -> Result<(), PlatformError> {
    let service_path = PathBuf::from("/etc/init.d")
        .join(&config.service_name);

    // Create SysVinit script
    let mut script_content = String::new();
    script_content.push_str("#!/bin/sh\n\n");
    script_content.push_str("### BEGIN INIT INFO\n");
    script_content.push_str(&format!("# Provides:          {}\n", config.service_name));
    script_content.push_str("# Required-Start:    $network $local_fs\n");
    script_content.push_str("# Required-Stop:     $network $local_fs\n");
    script_content.push_str("# Default-Start:     2 3 4 5\n");
    script_content.push_str("# Default-Stop:      0 1 6\n");
    script_content.push_str(&format!("# Short-Description: {}\n", config.description));
    script_content.push_str("### END INIT INFO\n\n");

    script_content.push_str(&format!("NAME=\"{}\"\n", config.service_name));
    script_content.push_str(&format!("DAEMON=\"{}\"\n", config.executable_path));

    if let Some(user) = &config.user {
        script_content.push_str(&format!("DAEMON_USER=\"{}\"\n", user));
    } else {
        script_content.push_str("DAEMON_USER=\"root\"\n");
    }

    if let Some(working_dir) = &config.working_directory {
        script_content.push_str(&format!("WORKING_DIR=\"{}\"\n", working_dir));
    }

    script_content.push_str(r#"
PIDFILE="/var/run/$NAME.pid"

# Exit if executable doesn't exist
[ -x "$DAEMON" ] || exit 5

# Load init function library
. /lib/lsb/init-functions

start() {
    log_daemon_msg "Starting $NAME"
    start-stop-daemon --start --quiet --background \
"#);

    if let Some(user) = &config.user {
        script_content.push_str(&format!("        --chuid {} \\\n", user));
    }

    if let Some(working_dir) = &config.working_directory {
        script_content.push_str(&format!("        --chdir {} \\\n", working_dir));
    }

    script_content.push_str(r#"        --make-pidfile --pidfile $PIDFILE \
        --exec $DAEMON
    log_end_msg $?
}

stop() {
    log_daemon_msg "Stopping $NAME"
    start-stop-daemon --stop --quiet --pidfile $PIDFILE
    log_end_msg $?
}

status() {
    status_of_proc -p $PIDFILE "$DAEMON" "$NAME"
}

case "$1" in
    start)
        start
        ;;
    stop)
        stop
        ;;
    restart)
        stop
        start
        ;;
    status)
        status
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}"
        exit 2
        ;;
esac

exit 0
"#);

    // Write file
    fs::write(&service_path, script_content)
        .map_err(|e| PlatformError::Io(e))?;

    // Make script executable
    fs::set_permissions(&service_path, fs::Permissions::from_mode(0o755))
        .map_err(|e| PlatformError::Io(e))?;

    println!("Registered SysVinit service at: {}", service_path.display());
    println!("To enable and start the service, run:");
    println!("sudo update-rc.d {} defaults", config.service_name);
    println!("sudo service {} start", config.service_name);

    Ok(())
}

fn register_openrc_service(config: &PlatformConfig) -> Result<(), PlatformError> {
    let service_path = PathBuf::from("/etc/init.d")
        .join(&config.service_name);

    // Create OpenRC script
    let mut script_content = String::new();
    script_content.push_str("#!/sbin/openrc-run\n\n");
    script_content.push_str(&format!("name=\"{}\"\n", config.description));
    script_content.push_str(&format!("description=\"{}\"\n", config.description));
    script_content.push_str(&format!("command=\"{}\"\n", config.executable_path));

    if let Some(user) = &config.user {
        script_content.push_str(&format!("command_user=\"{}\"\n", user));
    }

    if let Some(working_dir) = &config.working_directory {
        script_content.push_str(&format!("directory=\"{}\"\n", working_dir));
    }

    script_content.push_str("command_background=true\n");
    script_content.push_str("pidfile=\"/run/${RC_SVCNAME}.pid\"\n");
    script_content.push_str("\ndepend() {\n");
    script_content.push_str("\tneed net\n");
    script_content.push_str("}\n");

    // Write file
    fs::write(&service_path, script_content)
        .map_err(|e| PlatformError::Io(e))?;

    // Make script executable
    fs::set_permissions(&service_path, fs::Permissions::from_mode(0o755))
        .map_err(|e| PlatformError::Io(e))?;

    println!("Registered OpenRC service at: {}", service_path.display());
    println!("To enable and start the service, run:");
    println!("sudo rc-update add {} default", config.service_name);
    println!("sudo rc-service {} start", config.service_name);

    Ok(())
}

fn register_runit_service(config: &PlatformConfig) -> Result<(), PlatformError> {
    let service_dir = PathBuf::from("/etc/sv")
        .join(&config.service_name);

    // Create service directory
    fs::create_dir_all(&service_dir)
        .map_err(|e| PlatformError::Io(e))?;

    let run_script_path = service_dir.join("run");

    // Create run script
    let mut run_script = String::new();
    run_script.push_str("#!/bin/sh\n\n");
    run_script.push_str("exec 2>&1\n");

    if let Some(working_dir) = &config.working_directory {
        run_script.push_str(&format!("cd {}\n", working_dir));
    }

    if let Some(user) = &config.user {
        if let Some(group) = &config.group {
            run_script.push_str(&format!("exec chpst -u {}:{} {}\n", user, group, config.executable_path));
        } else {
            run_script.push_str(&format!("exec chpst -u {} {}\n", user, config.executable_path));
        }
    } else {
        run_script.push_str(&format!("exec {}\n", config.executable_path));
    }

    // Write run script
    fs::write(&run_script_path, run_script)
        .map_err(|e| PlatformError::Io(e))?;

    // Make script executable
    fs::set_permissions(&run_script_path, fs::Permissions::from_mode(0o755))
        .map_err(|e| PlatformError::Io(e))?;

    // Create symbolic link in /etc/service if it exists
    if Path::new("/etc/service").exists() {
        let target_link = PathBuf::from("/etc/service")
            .join(&config.service_name);

        if let Err(e) = std::os::unix::fs::symlink(&service_dir, &target_link) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(PlatformError::Io(e));
            }
        }
    }

    println!("Registered Runit service at: {}", service_dir.display());
    println!("To enable and start the service, run:");
    println!("sudo ln -s /etc/sv/{0} /var/service/{0}", config.service_name);

    Ok(())
}

// Helper function to check if a user exists and create if it doesn't
fn ensure_user_exists(user: &str, group: Option<&str>) -> Result<(), PlatformError> {
    // Check if user exists
    let user_exists = Command::new("id")
        .arg(user)
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(PlatformError::Io)?
        .success();

    if !user_exists {
        println!("Creating user '{}'...", user);

        // If group is specified and different from user, create it first if needed
        if let Some(group_name) = group {
            if group_name != user {
                let group_exists = Command::new("getent")
                    .args(["group", group_name])
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map_err(PlatformError::Io)?
                    .success();

                if !group_exists {
                    Command::new("groupadd")
                        .arg(group_name)
                        .status()
                        .map_err(PlatformError::Io)?;
                }
            }

            // Create user with specified group
            Command::new("useradd")
                .args([
                    "--system",
                    "--shell", "/sbin/nologin",
                    "--group", group_name,
                    user
                ])
                .status()
                .map_err(PlatformError::Io)?;
        } else {
            // Create user with default group
            Command::new("useradd")
                .args([
                    "--system",
                    "--shell", "/sbin/nologin",
                    user
                ])
                .status()
                .map_err(PlatformError::Io)?;
        }

        println!("User '{}' created successfully", user);
    }

    Ok(())
}

// Helper function to create directory and set permissions
fn ensure_directory_exists(dir: &str, user: Option<&str>, group: Option<&str>) -> Result<(), PlatformError> {
    let path = Path::new(dir);

    if !path.exists() {
        println!("Creating directory '{}'...", dir);
        fs::create_dir_all(path).map_err(PlatformError::Io)?;

        // Set directory ownership if user/group is specified
        if user.is_some() || group.is_some() {
            let user_arg = user.unwrap_or("root");
            let group_arg = group.unwrap_or("root");

            Command::new("chown")
                .args([&format!("{}:{}", user_arg, group_arg), dir])
                .status()
                .map_err(PlatformError::Io)?;
        }

        println!("Directory '{}' created successfully", dir);
    }

    Ok(())
}
