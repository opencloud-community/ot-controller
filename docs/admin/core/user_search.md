# User search

OpenTalk can search for users and display suggestions when attempting to
invite users into meetings. User search can be configured to behave differently
depending on the use case.

The HTTP endpoint that is used by frontend to query users is `GET /users/find`.

Currently the search configuration is not yet consolidated under a specific
configuration section, but instead spread over multiple sections. This is likely
to change in the future.

## Disabling the search endpoint entirely

The `/users/find` HTTP endpoint can be disabled entirely by setting
`disable_users_find = true` in the [`[endpoints]` section](endpoints.md).
The endpoint is enabled by default.

## Allow inviting users by email address

OpenTalk can be configured to allow inviting guests through external email
addresses, even if they can not register, e.g. because the instance is limited
to accounts of an organization. These guests will then receive an email with an
invite link they can use to join a meeting.

This behavior can be enabled by setting
`endpoints.event_invite_external_email_address` to `true` in the
[HTTP endpoints configuration](endpoints.md).

## Searching for users on Keycloak

The OpenTalk Controller can search on a Keycloak instance if configured
correctly. This does not happen through OIDC, but instead logs in to the
Keycloak web api and calls endpoints there.

If search on Keycloak is not configured, but the search HTTP endpoint is
enabled, then the OpenTalk Controller will search in its own
[database](database.md), where it only finds users that logged in to OpenTalk at
least once.

### Configuring Keycloak for user search

:::note

The Keycloak user interface changed in the past and because of that it's safe to assume
that it will continue to change moving forward. Instead of screenshots we describe what needs to be
done, and link to the Keycloak documentation where needed. These links
reference a specific version of Keycloak. If those settings are outdated, please refer to the
[Keycloak documentation archive](https://www.keycloak.org/documentation-archive.html)
and find the corresponding section there.

:::

1. Configure a client which will be used to access the Keycloak web api. Details
   about that can be found in the [Keycloak section](keycloak.md).
2. For the [OpenID Connect client](https://www.keycloak.org/docs/25.0.0/server_admin/index.html#proc-creating-oidc-client_server_administration_guide)
   enable **Service account roles** in the [Capability Config](https://www.keycloak.org/docs/25.0.0/server_admin/index.html#capability-config).
3. In the [Service account](https://www.keycloak.org/docs/25.0.0/server_admin/index.html#_service_accounts)
   add these service account roles:
   - `realm-management` *query-users*
   - `realm-management` *view-users*

### Configuring OpenTalk Controller for user search on Keycloak

1. Perform general [OIDC configuration](oidc.md#configuration).
2. Perform [user search configuration](#user-search-configuration), see below.
3. When attempting to invite users to a meeting, the suggestions should now
   contain Keycloak users that have not yet logged in to OpenTalk.

### User search configuration

In the past, configuration of OIDC and user search was done together within the [`keycloak`](keycloak_deprecated.md#deprecated-keycloak-configuration) section.
This is deprecated and should be replaced with the separate [`oidc`](oidc.md#configuration) and [`user_search`](#user-search-configuration) sections.

The section in the [configuration file](configuration.md) is called `user_search`.

| Field                             | Type     | Required | Default value                        | Description                                                                                                              |
| --------------------------------- | -------- | -------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| `backend`                         | `enum`   | yes      | -                                    | Defines which backend to use for user search. Must be `"keycloak_webapi"`                                                |
| `api_base_url`                    | `string` | yes      | -                                    | Base URL of the Keycloak web api                                                                                         |
| `client_id`                       | `string` | no       | From `oidc.controller.client_id`     | Client id that is used to authenticate against the user search API                                                       |
| `client_secret`                   | `string` | no       | From `oidc.controller.client_secret` | Client secret that is used to authenticate against the user search API                                                   |
| `external_id_user_attribute_name` | `string` | no       | See below                            | The attribute by which Keycloak and OpenTalk users are assigned to each other. See below for more details.               |
| `users_find_behavior`             | `enum`   | yes      | -                                    | Sets the behaviour of the `/users/find` endpoint. Either `"disabled"`, `"from_database"` or `"from_user_search_backend"` |

The `external_id_user_attribute_name` setting is used to configure how Keycloak users resulting from a search and registered
Opentalk users are assigned to each other.
The following assignment strategies are available:

- by Keycloak id (default): This is used if `external_id_user_attribute_name` is not set. Keycloak users are assigned to
  Opentalk users using Keycloak's id field.
- by user attribute: Keycloak must provide a user attribute holding the user IDs. The name of this user attribute must be
  set here in `external_id_user_attribute_name`.

The `users_find_behavior` setting configures the behaviour of the `/users/find` endpoint. This allows searching for users who have
not yet logged into the controller.
You can choose where to search for users or disable the endpoint completely for performance or privacy reasons.

### Examples

#### Default Setup

```toml
[user_search]
backend = "keycloak_webapi"
api_base_url = "https://localhost:8080/auth/admin/realms/OPENTALK"
client_id = "Controller"
client_secret = "v3rys3cr3t"
users_find_behavior = "from_user_search_backend"
```
