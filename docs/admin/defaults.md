---
sidebar_position: 111
---

# Default and fallback values

## Features

In the [configuration file](configuration.md), the format of a [`feature`](modules.md#features) is: `[<module>::]<feature>`.
A missing module specifier defaults to `"core"`. The features currently supported are:

- `core::call_in`
- `integration::outlook`

The [`modules`](modules.md#opentalk-controller-modules-subcommand) subcommand outputs all modules
available in the OpenTalk controller, including the features that can be enabled or disabled.

## Configuration

The section in the [configuration file](configuration.md) is called `defaults`.

| Field                              | Type       | Required | Default value | Description                                              |
| ---------------------------------- | ---------- | -------- | ------------- | -------------------------------------------------------- |
| `user_language`                    | `string`   | no       | `"en-US"`     | Default language of a new user                           |
| `screen_share_requires_permission` | `bool`     | no       | `false`       | When `true`, screen sharing requires explicit permission |
| `disabled_features`                | `string[]` | no       | `[]`          | A list of disabled features in the controller            |

### Examples

#### Disable the `core::call_in` and `integration::outlook` features

```toml
[defaults]
disabled_features = ["core::call_in", "integration::outlook"]
```

#### Set default user language, require explicit screen share permissions and disable the `core::call_in` feature

```toml
[defaults]
user_language = "de-DE"
screen_share_requires_permission = true
disabled_features = ["call_in"]
```
