---
sidebar_position: 104
title: Update Migration Guide
---

# Migration guide for updating to new versions

## General information

After installing/deploying the new version
[`opentalk-controller fix-acl`](../advanced/acl.md#opentalk-controller-fix-acl-subcommand)
must be run in order to update ACLs to match the newest version whenever
new endpoints were added for already present resources. However, even if no
endpoints were added, simply running the command does no harm.

## Updating to OpenTalk Controller `v0.26.0`

### Janus support removed entirely

Support for the Janus Media Server has been [removed entirely](../core/room_server.md)
in favor of [LiveKit](../core/livekit.md).

### Changes in the OIDC configuration

Configuration options of the authentication provider no longer target
[Keycloak](../core/keycloak_deprecated.md)
specifically, but rather [OIDC in general](../core/oidc.md). For the time being,
Keycloak is still the only supported authentication provider, but that is likely
to change in the future.

[User search](../core/user_search.md) needs to be configured separately now.

#### Example

Assuming this is the `[keycloak]` section in the
[controller configuration](../core/configuration.md).

```toml
[keycloak]
base_url = "https://accounts.example.com/auth"
realm = "MyRealm"
client_id = "Controller"
client_secret = "c64c5854-3f02-4728-a617-bbe98ec42b8f"
```

You must change that section to:

```toml
# Basic OIDC configuration
[oidc]
base_url = "https://accounts.example.com/auth/realms/MyRealm"

# Frontend configuration
[oidc.frontend]
# This is the id that the frontend client will use for authentication.
# Must be the same as configured by the `OIDC_CLIENT_ID` value in the
# web-frontend. See https://gitlab.opencode.de/opentalk/web-frontend
client_id = "Frontend"

# Controller configuration
[oidc.controller]
client_id = "Controller"
client_secret = "c64c5854-3f02-4728-a617-bbe98ec42b8f"

# User search is optional, needs extra configuration on the Keycloak server
[user_search]
backend = "keycloak_webapi"
api_base_url = "https://accounts.example.com/auth/admin/realms/MyRealm"
users_find_behavior = "from_user_search_backend"
```

## Updating to OpenTalk Controller `v0.25.0`

This controller version introduces support for [LiveKit](../core/livekit.md).
Support for the Janus Media Server [has been deprecated](../core/room_server.md).

See [the LiveKit migration documentation](./livekit.md).

## Updating to OpenTalk Controller `v0.16.0`

### Janus can now be connected to via websocket

The OpenTalk controller now can be configured to connect to Janus directly
via a websocket connection instead of using rabbitmq in-between.

See [the configuration documentation](../core/room_server.md).

## Updating to OpenTalk Controller `v0.15.0`

### Redis is only required for *clustered* operation

Since `v0.15.0` of the OpenTalk controller, the usage of a [Redis](../core/
redis.md) service is only required for synchronizing between nodes when the
controller should run in *clustered* mode. If the service should only be
provided by a single node, then the controller can run in *standalone* mode now
by removing the `[redis]` section from the configuration entirely.

## Updating to OpenTalk Controller `v0.9.0`

### Removal of server-side speaker detection

The audio level detection is no longer performed by the MCU (Multipoint Control
Unit, in this case the Janus Media Server), therefore the corresponding settings
are deprecated.

Deprecated settings in the [configuration file](../core/configuration.md) are:

- `room_server.speaker_focus_packets`
- `room_server.speaker_focus_level`

Corresponding entries should be removed from the configuration file. If one
of the settings is found in the configuration file, a warning is printed out as
information for the administrator.
