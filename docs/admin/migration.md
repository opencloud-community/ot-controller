---
sidebar_position: 103
---

# Migration guide for updating to new versions

## General information

After installing/deploying the new version
[`opentalk-controller fix-acl`](acl.md#opentalk-controller-fix-acl-subcommand)
must be run in order to update ACLs to match the newest version whenever
new endpoints were added for already present resources. However, even if no
endpoints were added, simply running the command does no harm.

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
