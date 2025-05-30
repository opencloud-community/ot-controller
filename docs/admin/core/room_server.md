# Janus Room Server

:::warning

The `room_server` configuration section described here was deprecated in the
OpenTalk Controller version 0.25.0, and is no longer supported in version 0.26.0
or later. The successor is [LiveKit](livekit.md).

:::

## Historic content before OpenTalk Controller v0.21.0 and older

These are no longer valid configuration options in the OpenTalk controller, but the information remains available for the time being, as the older versions are available when needed.

> OpenTalk organizes video and audio streams through [Janus](https://janus.conf.meetecho.com/) which is used as SFU (Selective Forwarding Unit) for the conferences. The communication between the OpenTalk controller and Janus goes through either a direct websocket connection or [RabbitMQ](rabbitmq.md). Multiple Janus instances can be configured for an OpenTalk deployment.
>
> The `room_server` section describes general settings that apply to all Janus instances that are used with the service. The individual Janus instances can be configured in the `room_server.connections` list field.
>
> ## Configuration
>
> The section in the [configuration file](configuration.md) is called `room_server`.
>
> ### Room Server section
>
> | Field                   | Type               | Required | Default value | Description                                                                                     |
> | ----------------------- | ------------------ | -------- | ------------- | ----------------------------------------------------------------------------------------------- |
> | `max_video_bitrate`     | `string`           | no       | 1500000       | The maximum bitrate for video sessions                                                          |
> | `max_screen_bitrate`    | `string`           | no       | 8000000       | The maximum bitrate for screen share sessions                                                   |
> | `speaker_focus_packets` | `int`              | no       | 50            | Number of packets with with given `speaker_focus_level` needed to detect a speaking participant |
> | `speaker_focus_level`   | `int`              | no       | 50            | Average value of audio level needed per packet. min: 127 (muted), max: 0 (loud)                 |
> | `connections`           | `list<Connection>` | no       | -             | List of connections to the room server, see below for more details                              |
>
> ### Room Server connections
>
> Connection settings for the channel used to talk to the room server.
>
> | Field              | Type     | Required | Default value | Description                                                                                                                                                                          |
> | ------------------ | -------- | -------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
> | `websocket_url`    | `string` | yes*     | -             | Websocket URL pointing towards a janus instance, e.g. `ws://my-janus-host:8188`                                                                                                      |
> | `to_routing_key`   | `string` | yes**    | -             | The routing key to send messages to the room server                                                                                                                                  |
> | `exchange`         | `string` | yes**    | -             | The exchange to use for communication with the room server                                                                                                                           |
> | `from_routing_key` | `string` | yes**    | -             | The routing key to receive messages from the room server                                                                                                                             |
> | `event_loops`      | `int`    | no       | -             | Number of event loops configured for this janus server. This value is used to balance new webrtc sessions on event loops. If its not provided, one event loop is spawned by default. |
>
> \* `websocket_url` is only required when connecting via websocket
> \*\* `to_routing_key`, `exchange`, `from_routing_key` are only required when using rabbitmq
> When both a `websocket_url` and the rabbitmq parameters are set, the websocket transport is used.
>
> ### Example
>
> ```toml
> [room_server]
> max_video_bitrate = "1500000"
> max_screen_bitrate = "8000000"
> speaker_focus_packets = "50"
> speaker_focus_level = "50"
>
> # Using a direct websocket connection
> [[room_server.connections]]
> websocket_url = "ws://my-janus-host:8188"
> event_loops = 92
>
> # Using a rabbitmq to connect to janus
> [[room_server.connections]]
> to_routing_key = "to-janus"
> exchange = "janus-exchange"
> from_routing_key = "from-janus"
> event_loops = 92
> ```
