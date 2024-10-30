# Identity Provider

This page describes how the OIDC provider is configured in the controller.
Generic information for Keycloak and its configuration can be found in the [Keycloak section](keycloak.md).

## Configuration

In the past, configuration of OIDC and user search was done together within the [`keycloak`](keycloak_deprecated.md#deprecated-keycloak-configuration) section.
Starting with controller version 0.21.0, this is deprecated, support will be removed in the future.
It should be replaced with the separate [`oidc`](#configuration) and [`user_search`](user_search.md#user-search-configuration) sections.

The section in the [configuration file](configuration.md) is called `oidc`.

| Field        | Type                                                  | Required | Default value | Description                                                                                                                                              |
| ------------ | ----------------------------------------------------- | -------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `authority`  | `string`                                              | yes      | -             | Base url for the OIDC authority. Will be used for frontend and controller unless overwritten by `oidc.frontend.authority` or `oidc.controller.authority` |
| `frontend`   | [frontend configuration](#frontend-configuration)     | yes      | -             | Configuration dedicated to the frontend                                                                                                                  |
| `controller` | [controller configuration](#controller-configuration) | yes      | -             | Configuration dedicated to the controller                                                                                                                |

### Frontend configuration

| Field           | Type     | Required | Default value         | Description                                                                      |
| --------------- | -------- | -------- | --------------------- | -------------------------------------------------------------------------------- |
| `authority`     | `string` | no       | From `oidc.authority` | OIDC authority base url for the frontend                                         |
| `client_id`     | `string` | yes      | -                     | Client id that will be used by the frontend when connecting to the oidc provider |

### Controller configuration

| Field           | Type     | Required | Default value         | Description                                                                            |
| --------------- | -------- | -------- | --------------------- | -------------------------------------------------------------------------------------- |
| `authority`     | `string` | no       | From `oidc.authority` | OIDC authority base url for the controller                                             |
| `client_id`     | `string` | yes      | -                     | Client id that will be used by the controller when connecting to the oidc provider     |
| `client_secret` | `string` | yes      | -                     | Client secret that will be used by the controller when connecting to the oidc provider |

### Examples

#### Default Setup

```toml
[oidc]
authority = "https://localhost:8080/auth/realms/OPENTALK"

[oidc.frontend]
client_id = "Frontend"

[oidc.controller]
client_id = "Controller"
client_secret = "v3rys3cr3t"
```

## OIDC and JWT

The following fields in the JWT returned by the OIDC provider are used by the OpenTalk Controller. These fields differ for authentication of normal users and services.

### JWT fields for user login

| Field           | Type       | Required                                                                      | Description                                                                                     |
| --------------- | ---------- | ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `exp`           | `string`   | yes                                                                           | RFC 3339 timestamp of the token's expiration date                                               |
| `iat`           | `string`   | yes                                                                           | RFC 3339 timestamp of the token's issuing date                                                  |
| `iss`           | `string`   | yes                                                                           | URL of the OIDC provider                                                                        |
| `sub`           | `string`   | yes                                                                           | Unique identifier of the user                                                                   |
| `email`         | `string`   | yes                                                                           | E-Mail address of the user                                                                      |
| `given_name`    | `string`   | yes                                                                           | The given name (also known as first name) of the user                                           |
| `family_name`   | `string`   | yes                                                                           | The family name (also know as last name) of the user                                            |
| `tenant_id`     | `string`   | if [tenant `assignment`](../advanced/tenants.md) is `"by_external_tenant_id"` | Contains the identifier of the user's tenant                                                    |
| `tariff_id`     | `string`   | if [tariffs](../advanced/tariffs.md) are used                                 | The external id of the tariff. See [tariffs](../advanced/tariffs.md) for further details        |
| `tariff_status` | `string`   | if [tariffs](../advanced/tariffs.md) are used                                 | The external id of the tariff status. See [tariffs](../advanced/tariffs.md) for further details |
| `x_grp`         | `string[]` | no                                                                            | A list of groups which the user is part of                                                      |
| `phone_number`  | `string`   | no                                                                            | The phone number of the user                                                                    |
| `nickname`      | `string`   | no                                                                            | Nickname of the user, typically used to prefill the display name of a meeting participant       |
| `picture`       | `string`   | no                                                                            | URL to a user picture, will replace the gravatar url generation for that user if provided       |

#### Security considerations

For the `picture` field, the frontend will download the images found under the
provided URL. Therefore it is important to only provide URLs that are guaranteed
to not inject unwanted content, but rather have a policy which ensures that only
valid images are served.

### JWT fields for service login

| Field           | Type          | Required | Description                                       |
| --------------- | ------------- | -------- | ------------------------------------------------- |
| `exp`           | `string`      | yes      | RFC 3339 timestamp of the token's expiration date |
| `iat`           | `string`      | yes      | RFC 3339 timestamp of the token's issuing date    |
| `iss`           | `string`      | yes      | URL of the OIDC provider                          |
| `realm_access`  | `RealmAccess` | yes      | An object containing realm access information     |

The `RealmAccess` object contains these fields:

| Field   | Type       | Required | Description                                                       |
| ------- | ---------- | -------- | ----------------------------------------------------------------- |
| `roles` | `string[]` | yes      | A list of role identifiers that the service is allowed to provide |

The list of known service roles is:

- `"opentalk-call-in"`: The service is allowed to provide a meeting [phone call-in service](../advanced/call_in.md).
- `"opentalk-recorder"`: The service is allowed to provide a meeting [recording service](../advanced/additional_services/recorder.md).
