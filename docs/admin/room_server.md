# Room Server

## Configuration

The section in the [configuration file](configuration.md) is called `room_server`.

### Room Server section

| Field                   | Type     | Required | Default value | Description                                                                                     |
| ----------------------- | -------- | -------- | ------------- | ----------------------------------------------------------------------------------------------- |
| `max_video_bitrate`     | `string` | no       | 1500000       | The maximum bitrate for video sessions                                                          |
| `max_screen_bitrate`    | `string` | no       | 8000000       | The maximum bitrate for screen share sessions                                                   |
| `speaker_focus_packets` | `int`    | no       | 50            | Number of packets with with given `speaker_focus_level` needed to detect a speaking participant |
| `speaker_focus_level`   | `int`    | no       | 50            | Average value of audio level needed per packet. min: 127 (muted), max: 0 (loud)                 |


### Room Server connections

The section in the [configuration file](configuration.md) is called `room_server`.

| Field              | Type     | Required | Default value | Description                                                                                                              |
| ------------------ | -------- | -------- | ------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `to_routing_key`   | `string` | yes      | -             | The routing key to send messages to the room server                                                                      |
| `exchange`         | `string` | yes      | -             | The exchange to use for communication with the room server                                                               |
| `from_routing_key` | `string` | yes      | -             | The routing key to receive messages from the room server                                                                 |
| `event_loops`      | `int`    | no       | -             | Number of event loops configured for this janus server. This value is used to balance new webrtc sessions on event loops |

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
