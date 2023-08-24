<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
SPDX-License-Identifier: EUPL-1.2
-->

# KeyCloak

The OpenTalk Controller uses [keycloak](https://www.keycloak.org/), an OpenID Connect compatible
identity and access management software for single sign-on.

## Configuration

The section in the [configuration file](configuration.md) is called `keycloak`.

| Field           | Type     | Required | Default value | Description                                                                                                                            |
| --------------- | -------- | -------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `base_url`      | `string` | yes      | -             | TCP port number where the Keycloak server can be reached                                                                               |
| `realm`         | `string` | yes      | -             | The name of the default Keycloak realm, read more on [Keycloak](https://www.keycloak.org/docs/latest/server_admin/#configuring-realms) |
| `client_id`     | `string` | yes      | -             | The unique identifier for the OpenTalk client                                                                                          |
| `client_secret` | `string` | yes      | -             | The secret corresponding to the specified client ID                                                                                    |

### Examples

#### Default Setup

```toml
[keycloak]
base_url = "http://localhost:8080/auth"
realm = "MyRealm"
client_id = "Controller"
client_secret = "c64c5854-3f02-4728-a617-bbe98ec42b8f"
```
