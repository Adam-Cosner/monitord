# monitord
A system monitoring daemon that can remotely report system metrics locally or over the network.

## Components
- **monitord**: The main daemon that collects and reports system metrics.
- **monitord-api**: A client library for interacting with the monitord daemon.
- **monitordctl**: The command-line interface for managing the monitord daemon.
- **monitord-types**: Data types to be transmitted between the daemon and clients.
