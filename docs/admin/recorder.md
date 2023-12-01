---
sidebar_position: 401
---

# OpenTalk Recorder

The OpenTalk Controller has the ability to establish communication with the
OpenTalk Recorder and initiate recording sessions. This interaction between the
OpenTalk Recorder and the OpenTalk Controller is facilitated through RabbitMQ.

## Configuration

To activate the recording functionality, it is necessary to configure the
`recording_task_queue` setting within the `rabbit_mq` section. Without this
configuration, the controller will not possess the capability to commence
recording sessions.

The section in the [configuration file](configuration.md) is called `rabbit_mq`.

### Usage Examples

#### Disabling Recording Capability in RabbitMQ

If you wish to disable the recording capability for the controller, simply
refrain from setting the `recording_task_queue` parameter in the configuration
file. Here's an example configuration without this setting:

```toml
[rabbit_mq]

# Other RabbitMQ settings ...

#recording_task_queue = "opentalk_recorder"
```

#### Enabling Recording Capability in RabbitMQ

To enable the recording capability for the controller, configure the
`recording_task_queue` parameter. This parameter should be set to the same value
as used for the recording process itself. Here's an example configuration that
enables recording:

```toml
[rabbit_mq]

# Other RabbitMQ settings ...

recording_task_queue = "opentalk_recorder"
```
