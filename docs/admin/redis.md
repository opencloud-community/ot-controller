<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
SPDX-License-Identifier: EUPL-1.2
-->

# Redis

The OpenTalk controller uses the in-memory data store [Redis](https://redis.com/) for caching data related to a video conference session.

## Configuration

The section in the [configuration file](configuration.md) is called `redis`.

| Field      | Type     | Required | Default value             | Description                                                                                                                            |
| ---------- | -------- | -------- | ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `url`      | `string` | no       | "redis://localhost:6379/" | TCP port number where the Redis server can be reached                                                                               |

### Examples

#### Default Setup

The default setup requires no manual input:

```toml
[redis]
#url = "redis://localhost:6379/"
```

#### Custom Redis URL

A custom Redis URL can be specified as following:

```toml
[redis]
url = "redis://localhost:6399/"
```
