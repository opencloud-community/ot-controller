# Modules

Modules provide functionality that can be used inside meetings.

## Features

A module might have features that can be enabled or disabled in the
[configuration file](../core/configuration.md) or using [tariffs](tariffs.md). The
features available can be obtained using the
[`modules`](#opentalk-controller-modules-subcommand) subcommand.

## Available modules

## `opentalk-controller modules` subcommand

This command outputs all modules available in the OpenTalk controller, including
the features that can be enabled or disabled:

```text
opentalk-controller modules list
```

Example output for the community edition:

<!-- begin:fromfile:cli-usage/opentalk-controller-modules-list.md -->

```text
echo: []
breakout: []
moderation: []
recording: ["record", "stream"]
recording_service: []
core: ["call_in"]
chat: []
automod: []
integration: ["outlook"]
livekit: []
polls: []
meeting_notes: []
shared_folder: []
timer: []
whiteboard: []
meeting_report: []
subroom_audio: []
training_participation_report: []
```

<!-- end:fromfile:cli-usage/opentalk-controller-modules-list.md -->
