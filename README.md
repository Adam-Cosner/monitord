<h1 style="text-align: center">MonitorD</h1>
<p style="text-align: center">An in-development system monitoring daemon for Linux that can locally or remotely fetch system statistics and deliver them to connected clients.</p>
<h2 style="text-align:center">Components</h2>
<h3><a href="./monitord/README.md">monitord</a>: The Main Binary</h3>
<p>This component is the core binary of the daemon that is set up as a service on the monitored system. It operates via a subscription model, where clients will initiate a connection. The daemon will keep an internal registry of connected clients and concurrently collect the metrics on the requested interval and push to clients.</p>
<h3><a href="./monitordctl/README.md">monitordctl</a>: The Control Utility</h3>
<p>This component is the control utility that is installed alongside the daemon that allows for runtime administration of the daemon. Only operates on locally installed daemon instances.</p>
<h3><a href="./monitord-client/README.md">monitord-client</a>: Client Library</h3>
<p>This component provides the client-facing interface for initiating a connection and receiving the metrics from the target daemon.</p>
<h3><a href="./monitord-metrics/README.md">monitord-metrics</a>: Metric Collection Library</h3>
<p>Contains the definition of the metric collection logic for others to use.</p>
