---
sidebar_position: 304
---

# Report generator

The [OpenTalk Report generator](https://gitlab.opencode.de/opentalk/terdoc) is a service whose main purpose is to generate reports (e.g. in PDF).

When no Reports generator is configured, it is disabled.

## Deploy Report generator

The report generator has to be configured and started separately.

You can clone the repository and build the Docker image yourself or download the image directly from the [Container Registry](https://gitlab.opencode.de/opentalk/terdoc/container_registry).

## Configuration

The section in the [configuration file](../../core/configuration.md) is called `reports`.

| Field     | Type     | Required | Default value | Description                                                |
| --------- | -------- | -------- | ------------- | ---------------------------------------------------------- |
| `url`     | `string` | yes      | -             | The URI address where the reports generator can be reached |

### Examples

#### Default Setup

```toml
[reports]
url = "http://localhost:8085"
```
