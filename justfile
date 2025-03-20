# monitord justfile

# Default recipe to show available commands
default:
    @just --list

# Build the entire workspace in release mode
build:
    cargo build --release --workspace

# Install monitord service to system
install: build
    @echo "Installing monitord service..."
    sudo install -Dm0755 target/release/monitord /usr/local/bin/
    @echo "Installation complete. Run 'just register-service' to register as a system service."

# Uninstall monitord service from system
uninstall:
    @echo "Uninstalling monitord service..."
    # First stop and disable service if registered
    @if systemctl is-active --quiet monitord; then \
        sudo systemctl stop monitord; \
    fi
    @if systemctl is-enabled --quiet monitord; then \
        sudo systemctl disable monitord; \
    fi
    @if [ -f /etc/systemd/system/monitord.service ]; then \
        sudo rm /etc/systemd/system/monitord.service; \
        sudo systemctl daemon-reload; \
    fi
    @if [ -f /etc/init.d/monitord ]; then \
        sudo rm /etc/init.d/monitord; \
        if command -v update-rc.d > /dev/null; then \
            sudo update-rc.d monitord remove; \
        fi; \
    fi
    @if [ -d /etc/sv/monitord ]; then \
        sudo rm -rf /etc/sv/monitord; \
        if [ -L /etc/service/monitord ]; then \
            sudo rm /etc/service/monitord; \
        fi; \
        if [ -L /var/service/monitord ]; then \
            sudo rm /var/service/monitord; \
        fi; \
    fi
    # Remove binary
    sudo rm -f /usr/local/bin/monitord
    @echo "Uninstall complete."

# Register monitord as a system service
register-service *ARGS:
    @echo "Registering monitord as a system service..."
    sudo /usr/local/bin/monitord --register-service {{ARGS}}
    @echo "Registration complete. Check service status with 'just status'."

# Check the status of monitord service
status:
    @echo "Checking monitord service status..."
    @if command -v systemctl > /dev/null && systemctl is-active --quiet monitord 2>/dev/null; then \
        systemctl status monitord; \
    elif [ -f /etc/init.d/monitord ] && command -v service > /dev/null; then \
        service monitord status; \
    elif [ -d /etc/sv/monitord ] && command -v sv > /dev/null; then \
        sv status monitord; \
    else \
        echo "monitord service not found or not registered."; \
    fi

# Start the monitord service
start:
    @echo "Starting monitord service..."
    @if command -v systemctl > /dev/null && [ -f /etc/systemd/system/monitord.service ]; then \
        sudo systemctl start monitord; \
    elif [ -f /etc/init.d/monitord ] && command -v service > /dev/null; then \
        sudo service monitord start; \
    elif [ -d /etc/sv/monitord ] && command -v sv > /dev/null; then \
        sudo sv start monitord; \
    else \
        echo "monitord service not found. Run 'just register-service' first."; \
    fi

# Stop the monitord service
stop:
    @echo "Stopping monitord service..."
    @if command -v systemctl > /dev/null && systemctl is-active --quiet monitord 2>/dev/null; then \
        sudo systemctl stop monitord; \
    elif [ -f /etc/init.d/monitord ] && command -v service > /dev/null; then \
        sudo service monitord stop; \
    elif [ -d /etc/sv/monitord ] && command -v sv > /dev/null; then \
        sudo sv stop monitord; \
    else \
        echo "monitord service not running or not found."; \
    fi

# Restart the monitord service
restart:
    @echo "Restarting monitord service..."
    @if command -v systemctl > /dev/null && [ -f /etc/systemd/system/monitord.service ]; then \
        sudo systemctl restart monitord; \
    elif [ -f /etc/init.d/monitord ] && command -v service > /dev/null; then \
        sudo service monitord restart; \
    elif [ -d /etc/sv/monitord ] && command -v sv > /dev/null; then \
        sudo sv restart monitord; \
    else \
        echo "monitord service not found. Run 'just register-service' first."; \
    fi

# Run development version for testing
run-dev:
    cargo run --bin monitord

# Run with debug logging
run-debug:
    RUST_LOG=debug cargo run --bin monitord

# Clean build artifacts
clean:
    cargo clean
