# Monitord Service

The monitoring daemon service component.

## Service Registration

The `monitord` service can be registered as a system service on Linux with different init systems.

### Register as a system service

```bash
# Register with automatic init system detection
sudo /usr/bin/monitord --register-service

# Register with specific init system type
sudo /usr/bin/monitord --register-service --init=systemd
sudo /usr/bin/monitord --register-service --init=sysvinit
sudo /usr/bin/monitord --register-service --init=openrc
sudo /usr/bin/monitord --register-service --init=runit

# Use custom parameters
sudo /usr/bin/monitord --register-service \
  --name=my-monitord \
  --description="Custom monitoring daemon" \
  --path=/opt/monitord/bin/monitord \
  --user=monitord \
  --group=monitord \
  --workdir=/opt/monitord
```

### Supported init systems

- **SystemD**: Modern init system used by most major Linux distributions like Ubuntu, Fedora, CentOS, Debian, etc.
- **SysVInit**: Traditional init system used in older Linux distributions
- **OpenRC**: Init system used by Gentoo, Alpine, and others
- **Runit**: Init system focused on service supervision, used by Void Linux and as optional in some other distributions

### After registration

Depending on the init system, you'll need to enable and start the service:

**SystemD**:
```bash
sudo systemctl enable --now monitord
```

**SysVInit**:
```bash
sudo update-rc.d monitord defaults
sudo service monitord start
```

**OpenRC**:
```bash
sudo rc-update add monitord default
sudo rc-service monitord start
```

**Runit**:
```bash
sudo ln -s /etc/sv/monitord /var/service/monitord
```

## Configuration

Configuration can be provided via environment variables or a config file.