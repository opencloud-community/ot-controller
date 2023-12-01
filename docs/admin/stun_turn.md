---
sidebar_position: 115
---

# STUN and TURN

The backend provides an endpoint that offers information about the STUN and
TURN servers. STUN URIs are simply a list of usable STUN endpoints that can
be used by the client. The TURN server utilizes the `pre_shared_key` from
the configuration to generate credentials. All STUN and TURN credentials and
endpoints can be found in a list under `/turn`.

## Configuration

The sections in the [configuration file](configuration.md) are called `stun` and `turn.

### STUN section

| Field  | Type           | Required | Default value | Description                   |
| ------ | -------------- | -------- | ------------- | ----------------------------- |
| `stun` | `list<string>` | no       | -             | List of STUN server endpoints |

### TURN section

| Field      | Type               | Required | Default value | Description                                                             |
| ---------- | ------------------ | -------- | ------------- | ----------------------------------------------------------------------- |
| `lifetime` | `number`           | no       | 86400         | Lifetime of the generated credentials in seconds                        |
| `servers`  | `list<TurnServer>` | no       | -             | List of TURN server configurations, see TURN server configuration below |

#### The configuration for a TurnServer

| Field            | Type           | Required | Default value | Description                                         |
| ---------------- | -------------- | -------- | ------------- | --------------------------------------------------- |
| `uris`           | `list<string>` | no       | -             | List of TURN server endpoints                       |
| `pre_shared_key` | `string`       | yes      | -             | The pre-shared key to generate the TURN credentials |

### Examples

#### Custom Turn Server

```toml
[turn]
# Lifetime of the generated credentials (in seconds)
lifetime = 86400

[[turn.servers]]
# URIS of this Turn Server following rfc7065
uris = [
    "turn:127.0.0.1:3478?transport=udp",
    "turn:127.0.0.1:3478?transport=tcp",
    "turns:127.0.0.1:5349?transport=tcp"
]
# The Pre Shared Key set with --static-auth-secret=...
pre_shared_key = "opentalk2"
```

#### Custom Stun Server

```toml
[stun]
uris = ["stun:127.0.0.1:3478"]
```
