# Room Server

OpenTalk organizes video and audio streams through [Janus](https://janus.conf.meetecho.com/) which is used as SFU (Selective Forwarding Unit) for the conferences. The communication between the OpenTalk controller and Janus goes through [RabbitMQ](rabbitmq.md). Multiple Janus instances can be configured for an OpenTalk deployment.

The `room_server` section describes general settings that apply to all Janus instances that are used with the service. The individual Janus instances can be configured in the `room_server.connections` list field.

## Configuration

The section in the [configuration file](configuration.md) is called `room_server`.

### Room Server section

| Field                   | Type               | Required | Default value | Description                                                                                     |
| ----------------------- | ------------------ | -------- | ------------- | ----------------------------------------------------------------------------------------------- |
| `max_video_bitrate`     | `string`           | no       | 1500000       | The maximum bitrate for video sessions                                                          |
| `max_screen_bitrate`    | `string`           | no       | 8000000       | The maximum bitrate for screen share sessions                                                   |
| `speaker_focus_packets` | `int`              | no       | 50            | Number of packets with with given `speaker_focus_level` needed to detect a speaking participant |
| `speaker_focus_level`   | `int`              | no       | 50            | Average value of audio level needed per packet. min: 127 (muted), max: 0 (loud)                 |
| `connections`           | `list<Connection>` | no       | -             | List of connections to the room server, see below for more details                              |

### Room Server connections

Connection settings for the channel used to talk to the room server.

| Field              | Type     | Required | Default value | Description                                                                                                                                                                          |
| ------------------ | -------- | -------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `to_routing_key`   | `string` | yes      | -             | The routing key to send messages to the room server                                                                                                                                  |
| `exchange`         | `string` | yes      | -             | The exchange to use for communication with the room server                                                                                                                           |
| `from_routing_key` | `string` | yes      | -             | The routing key to receive messages from the room server                                                                                                                             |
| `event_loops`      | `int`    | no       | -             | Number of event loops configured for this janus server. This value is used to balance new webrtc sessions on event loops. If its not provided, one event loop is spawned by default. |

### Example

```toml
[room_server]
max_video_bitrate = "1500000"
max_screen_bitrate = "8000000"
speaker_focus_packets = "50"
speaker_focus_level = "50"

[[room_server.connections]]
to_routing_key = "to-janus"
exchange = "janus-exchange"
from_routing_key = "from-janus"
event_loops = 92
```
