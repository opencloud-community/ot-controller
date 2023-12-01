---
sidebar_position: 108
title: Modules that can be used in meetings
---

# Modules

Modules provide functionality that can be used inside meetings.

## Features

A module might have features that can be enabled or disabled in the
[configuration file](configuration.md) or using [tariffs](tariffs.md). The
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

<!-- begin:fromfile:text:cli-usage/opentalk-controller-modules-list -->

```text
echo: []
breakout: []
moderation: []
recording: []
core: ["call_in"]
chat: []
integration: ["outlook"]
media: []
polls: []
protocol: []
shared_folder: []
timer: []
whiteboard: []
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-modules-list -->
