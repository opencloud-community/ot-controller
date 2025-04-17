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

## Unreleased

### Change to the OpenID Connect Integration

The OpenID Connect integration has been changed to support more authentication
flows for clients. Previously, ID tokens were used to synchronize user information
from the OpenID Provider with the OpenTalk database. Now, the `userinfo` endpoint
is used to fetch user information directly from the OpenID Provider, removing the
need for the ID token.

OpenID Providers (e.g. Keycloak) must be updated to include the [documented fields](../core/oidc.md#oidc-and-user-info) in the `userinfo` endpoint instead of the ID token.

## Updating to OpenTalk Controller `v0.29.0`

### Removal of the reports configuration and the `terdoc` service

The `terdoc` service for generating reports, e.g. in PDF format, has showed
several flaws. It requires an installation of LibreOffice in the container,
which cannot be operated properly in some environments with strict security
requirements.

Therefore the `terdoc` service is history now, and we use the
[`typst`](https://typst.app/) library internally for generating PDF reports.

For the time being, the style and format of these reports is baked in and
requires no further configuration. Therefore the `[report]`  section in the
configuration is currently deprecated and might be reintroduced in the future.
If a `[report]` section is present in the configuration file, a warning will be
shown when the OpenTalk Controller is started.

Further details about the current state of report creation are available in the
[Meeting Reports documentation](../core/meeting_reports.md).

### Removal of the TURN configuration and endpoint

With the transition from Janus to Livekit, the TURN configuration (in form of
the [`[turn]` section in the configuration file](../core/stun_turn.md) and the
`/turn` endpoint of the controller are deprecated and will be removed in the
future. TURN servers are managed and configured directly in LiveKit.

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
