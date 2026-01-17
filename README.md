<h1 align="center">MonitorD</h1>
<p align="center">An in-development system monitoring daemon for Linux that can locally or remotely fetch system statistics and deliver them to connected clients.</p>
<h2 align="center">Components</h2>
<h3><a href="./monitord/README.md">monitord</a>: The Main Binary</h3>
<p>This component is the core binary of the daemon that is set up as a systemd service on the monitored system. Clients initiate a connection to the daemon and receive a stream of system snapshots. The daemon will keep an internal registry of connected clients and concurrently collect the metrics on the requested interval and push to clients.</p>
<h3><a href="./monitordctl/README.md">monitordctl</a>: The Control Utility</h3>
<p>This component is the control utility that is installed alongside the daemon that allows for runtime administration. This utility only operates on instances on the current machine. Allows for enabling TCP connections, listing authorized clients, and setting connection timeouts.</p>
<h3><a href="./monitord-client/README.md">monitord-client</a>: Client Library</h3>
<p>This component provides a client-facing async library for initiating a connection and receiving the metrics from the target daemon.</p>
<h3><a href="./monitord-metrics/README.md">monitord-metrics</a>: Metric Collection Library</h3>
<p>Contains the core system metric collection logic, allowing it to be used separately by other projects.</p>
