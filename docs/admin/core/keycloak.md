# Identity Provider (Keycloak)

The OpenTalk Controller uses [Keycloak](https://www.keycloak.org/), an OpenID Connect compatible
identity and access management software for single sign-on.

## Configuring Keycloak for OpenTalk Controller

:::note

The Keycloak user interface changed in the past and because of that it's safe to assume
that it will continue to change moving forward. Instead of screenshots we describe what needs to be
done, and link to the Keycloak documentation where needed. These links
reference a specific version of Keycloak. If those settings are outdated, please refer to the
[Keycloak documentation archive](https://www.keycloak.org/documentation-archive.html)
and find the corresponding section there.

:::

This manual describes the configuration for the OpenTalk Controller only, other
OpenTalk components might need separate configuration.

1. Create a [realm](https://www.keycloak.org/docs/25.0.0/server_admin/index.html#proc-creating-a-realm_server_administration_guide) for usage with OpenTalk if it hasn't been created yet.
   - The **Realm ID** will be used in the `keycloak.realm` configuration field.
2. Create an [OpenID Connect client](https://www.keycloak.org/docs/25.0.0/server_admin/index.html#proc-creating-oidc-client_server_administration_guide).
   - The **Client ID**  will be used in the `keycloak.client_id` configuration field.
   - Enable **Client authentication** and **Service account roles** in the [Capability Config](https://www.keycloak.org/docs/25.0.0/server_admin/index.html#capability-config).
3. Create [Confidential client credentials](https://www.keycloak.org/docs/25.0.0/server_admin/index.html#_client-credentials).
   - Use the Client Authenticator **Client Id and Secret** .
   - The **Client secret** will be used in the `keycloak.client_secret` configuration field.

## Controller configuration

In the past, the OIDC and user search section in the [configuration file](configuration.md) was called [`keycloak`](keycloak_deprecated.md#deprecated-keycloak-configuration).
Starting with controller version 0.21.0, this is deprecated, support will be removed in the future.
Use the separate [`oidc`](oidc.md#configuration) and [`user_search`](user_search.md#user-search-configuration) sections instead.
