---
sidebar_position: 104
title: Update Migration Guide
---

# Migration guide for updating to new versions

## General information

After installing/deploying the new version
[`opentalk-controller fix-acl`](advanced/acl.md#opentalk-controller-fix-acl-subcommand)
must be run in order to update ACLs to match the newest version whenever
new endpoints were added for already present resources. However, even if no
endpoints were added, simply running the command does no harm.

## Updating to OpenTalk Controller v0.15.0

### Redis is only required for *clustered* operation

Since `v0.15.0` of the OpenTalk controller, the usage of a [Redis](core/
redis.md) service is only required for synchronizing between nodes when the
controller should run in *clustered* mode. If the service should only be
provided by a single node, then the controller can run in *standalone* mode now
by removing the `[redis]` section from the configuration entirely.

## Updating to OpenTalk Controller v0.9.0

### Removal of server-side speaker detection

The audio level detection is no longer performed by the MCU (Multipoint Control
Unit, in this case the Janus Media Server), therefore the corresponding settings
are deprecated.

Deprecated settings in the [configuration file](configuration.md) are:

- `room_server.speaker_focus_packets`
- `room_server.speaker_focus_level`

Corresponding entries should be removed from the configuration file. If one
of the settings is found in the configuration file, a warning is printed out as
information for the administrator.
