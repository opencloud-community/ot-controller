---
sidebar_position: 100
title: Migrate to LiveKit
---

# Migrate to LiveKit

Migrating from Janus to LiveKit involves several key steps to ensure a smooth transition. Follow the steps outlined below to successfully migrate your services.

1. **Shutdown and Remove Janus Resources**
    Begin by shutting down your existing Janus services and removing any associated resources to prevent conflicts during the migration process.

2. **Configure and Start LiveKit Server**
    - Use the container image `livekit/livekit-server:v1.7` in combination with controller version `v0.25.0`.
    - If you are using Docker, it is recommended to run LiveKit in `network_mode: host`. This is crucial because:
        - A large number of UDP ports are required for proper functionality.
        - Not using `network_mode: host` will result in a separate process being started for each mapped UDP port.
        - Ensure that the UDP port range (e.g. 20000-25000 or the range that was previously used for janus) is reachable for all users.

    - Set the `--node-ip` to specify the IP of the host, which will be advertised to clients. Alternatively, you can set `rtc.use_external_ip` to true to allow LiveKit to discover the true IP using STUN in cloud environments where the host has a public IP address but is not directly exposed.

    - Configure the access tokens used by the controller and livekit. In the livekit configuration, add an entry in the format `<NAME>: <SECRET>`, ensuring it is in sync with the controller settings for `api_key` and `api_secret`.

    Here is a sample configuration for the LiveKit server:

    ```yaml
    ---
    port: 7880
    rtc:
      tcp_port: 7881
      port_range_start: 20000
      port_range_end: 25000
      use_external_ip: true
    keys:
      controller_key: -secret-
    logging:
      json: false
      level: info
    ```

3. **Adjust Reverse Proxy Configuration**
    If you plan to host LiveKit behind a reverse proxy, ensure that:
    - The reverse proxy handles the TLS encryption.
    - Port 7880 (or the configured port) is set as the target for the reverse proxy.

4. **Adjust Controller Configuration**
    - Remove the `[room_server]` configuration from your controller settings.
    - Set up LiveKit in your controller configuration as follows:

    ```toml
    [livekit]
    api_key = "controller_key"
    api_secret = "-secret-"
    public_url = "https://livekit-public.example.com"
    service_url = "https://livekit-internal.example.com"
    ```

    - Ensure that `public_url` is reachable for all users connecting to LiveKit.
    - The `service_url` should be the URL where the controller can access the LiveKit server, which can be an internal routed URL or IP.
    - The `api_key` and `api_secret` must match the values configured in the LiveKit server.
    - The recorder service will fetch the configuration from the controller

5. **Frontend Configuration**
    The frontend needs an additional environment variable `LIVEKIT_SERVER_URL`
    - This should be the same as the `public_url` set in the controller
    - e.g. `LIVEKIT_SERVER_URL=https://livekit-public.example.com`

## Known Issues

- The Obelisk-Service does only support basic audio codecs and has no video support yet
- Users may experience connectivity issues with Firefox when connecting to a Room via a VPN under certain circumstances.
- LiveKit cannot be hosted behind a URL with a path; a dedicated subdomain is required.
