# HTTP Endpoints

The behavior of some API endpoints of the OpenTalk Controller can be modified.

## Configuration

The section in the [configuration file](configuration.md) is called `endpoints`.

| Field                                 | Type   | Required | Default value | Description                                                                                                                                                                                                                                                   |
| ------------------------------------- | ------ | -------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `disable_users_find`                  | `bool` | no       | false         | Disables the `GET /users/find` endpoint completely. The endpoint will return a `404 Not Found` response when this is enabled. This is deprecated, replace with `user_search.users_find_behavior`.                                                             |
| `users_find_use_kc`                   | `bool` | no       | false         | Use [Keycloaks](keycloak.md) user database in the user search for the `GET /users/find` endpoint. Search results may include users that were never registered on the OpenTalk Controller. This is deprecated, replace with `user_search.users_find_behavior`. |
| `event_invite_external_email_address` | `bool` | no       | false         | Affects the `POST /events/{event_id}/invites` endpoint and allows users to invite email addresses that are unknown to the Controller or Keycloak.                                                                                                             |
| `disallow_custom_display_name`        | `bool` | no       | false         | Enforces the display name that was provided by Keycloak and disallows users to change their display names via the `PATCH /users/me` endpoint.                                                                                                                 |
| `disable_openapi`                     | `bool` | no       | false         | Disables the `GET /v1/openapi.json` and `GET /swagger` endpoints which serve information about the OpenTalk controller WebAPI.                                                                                                                                |

For configuring user search, see the [User search section](user_search.md).

### Examples

#### Default Setup

```toml
[endpoints]
event_invite_external_email_address = false
disallow_custom_display_name = false
disable_openapi = false
```
