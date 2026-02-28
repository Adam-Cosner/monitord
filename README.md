<h1 style="text-align: center;">
MonitorD
</h1>
<p style="text-align: center;">
An in-development system monitoring daemon for Linux that can locally or remotely fetch system statistics and deliver them to connected clients.
</p>

---

# Components

- ### __monitordctl__: The control utility
- ### __monitord__: The service binary
- ### __monitord::collector__: The statistic collection backend
- ### __monitord::metrics__: The data type definitions
- ### __monitord::client__: The client library
