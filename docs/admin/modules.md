<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
SPDX-License-Identifier: EUPL-1.2
-->

# Modules

Modules provide functionality that can be used inside meetings.

## Available modules

This command outputs all modules available in the OpenTalk controller, including
the features that can be enabled or disabled, e.g. using [tariffs](tariffs.md):

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
