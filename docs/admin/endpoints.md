---
sidebar_position: 112
---

# Endpoints

The behavior of some API endpoints of the OpenTalk Controller can be modified.

## Configuration

The section in the [configuration file](configuration.md) is called `endpoints`.

| Field                                 | Type   | Required | Default value | Description                                                                                                                                                                              |
| ------------------------------------- | ------ | -------- | ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `disable_users_find`                  | `bool` | no       | false         | Disables the `GET /users/find` endpoint completely. The endpoint will return a `404 Not Found` response when this is enabled.                                                            |
| `users_find_use_kc`                   | `bool` | no       | false         | Use [Keycloaks](keycloak.md) user database in the user search for the `GET /users/find` endpoint. Search results may include users that were never registered on the OpenTalk Controller |
| `event_invite_external_email_address` | `bool` | no       | false         | Affects the `POST /events/{event_id}/invites` endpoint and allows users to invite email addresses that are unknown to the Controller or Keycloak.                                        |
| `disallow_custom_display_name`        | `bool` | no       | false         | Enforces the display name that was provided by Keycloak and disallows users to change their display names via the `PATCH /users/me` endpoint.                                            |

### Examples

#### Default Setup

```toml
[endpoints]
disable_users_find = false
users_find_use_kc = false
event_invite_external_email_address = false
disallow_custom_display_name = false
```
