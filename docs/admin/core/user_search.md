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

The Keycloak user interface changed in the past, and might change in the future,
therefore we don't include any screenshots here. Instead we describe what needs
to be done, and link to the Keycloak documentation where needed. These links
reference a specific version of Keycloak, if you are on a different version,
look it up in the
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

1. Perform general [Keycloak configuration](keycloak.md#configuration).
2. Configure the [HTTP enpdoint](endpoints.md):
   - Set `endpoints.disable_users_find` to `false`
   - Set `endpoints.users_find_use_kc` to `true`
3. When attempting to invite users to a meeting, the suggestions should now
   contain Keycloak users that have not yet logged in to OpenTalk.
