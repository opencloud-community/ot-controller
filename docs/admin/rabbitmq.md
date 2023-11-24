---
sidebar_position: 201
---

# RabbitMQ

## Configuration

The section in the [configuration file](configuration.md) is called `rabbit_mq`.

| Field                  | Type     | Required | Default value                       | Description                                                      |
| ---------------------- | -------- | -------- | ----------------------------------- | ---------------------------------------------------------------- |
| `url`                  | `string` | no       | "amqp://guest:guest@localhost:5672" | The RabbitMQ broker URL connection                               |
| `mail_task_queue`      | `string` | no       | -                                   | Name of the RabbitMQ queue for the [SMTP-Mailer](smtp_mailer.md) |
| `recording_task_queue` | `string` | no       | -                                   | Name of the RabbitMQ queue for the [recorder](recorder.md)       |
| `min_connections`      | `uint`   | no       | 10                                  | Minimum connections to retain when removing stale connections    |
| `max_channels`         | `uint`   | no       | 100                                 | Maxmimum number of AMQP channels per connection allowed          |

### Examples

#### With Mail Worker Queue

```toml
[rabbit_mq]
mail_task_queue = "opentalk_mailer"
```

#### With All Configurations

```toml
[rabbit_mq]
url = "amqp://username:password@host/%2F"
mail_task_queue = "opentalk_mailer"
recording_task_queue = "opentalk_recorder"
min_connections = 10
max_channels_per_connection = 100
```
