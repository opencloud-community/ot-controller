---
sidebar_position: 203
---

# KeyCloak

The OpenTalk Controller uses [keycloak](https://www.keycloak.org/), an OpenID Connect compatible
identity and access management software for single sign-on.

## Configuration

The section in the [configuration file](configuration.md) is called `keycloak`.

| Field                               | Type     | Required | Default value | Description                                                                                                                            |
| ------------------------------------| -------- | -------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `base_url`                          | `string` | yes      | -             | TCP port number where the Keycloak server can be reached                                                                               |
| `realm`                             | `string` | yes      | -             | The name of the default Keycloak realm, read more on [Keycloak](https://www.keycloak.org/docs/latest/server_admin/#configuring-realms) |
| `client_id`                         | `string` | yes      | -             | The unique identifier for the OpenTalk client                                                                                          |
| `client_secret`                     | `string` | yes      | -             | The secret corresponding to the specified client ID                                                                                    |
| `external_id_user_attribute_name`   | `string` | no       | See below     | The attribute by which Keycloak and OpenTalk users are assigned to each other. See below for more details.                             |

The `external_id_user_attribute_name` setting is used to configure how Keycloak users resulting from a search and registered
Opentalk users are assigned to each other.
The following assignment strategies are available:

- by Keycloak id (default): This is used if `external_id_user_attribute_name` is not set. Keycloak users are assigned to
  Opentalk users using Keycloak's id field.
- by user attribute: Keycloak must provide a user attribute holding the user IDs. The name of this user attribute must be
  set here in `external_id_user_attribute_name`.

### Examples

#### Default Setup

```toml
[keycloak]
base_url = "http://localhost:8080/auth"
realm = "MyRealm"
client_id = "Controller"
client_secret = "c64c5854-3f02-4728-a617-bbe98ec42b8f"
```

## KeyCloak setup

KeyCloak can provide some fields in the JWT to the OpenTalk Controller. These fields differ for authentication of normal users and services.

### JWT fields for user login

| Field           | Type       | Required                                                          | Description                                                                               |
| --------------- | ---------- | ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `exp`           | `string`   | yes                                                               | RFC 3339 timestamp of the token's expiration date                                         |
| `iat`           | `string`   | yes                                                               | RFC 3339 timestamp of the token's issuing date                                            |
| `iss`           | `string`   | yes                                                               | URL of the OIDC provider                                                                  |
| `sub`           | `string`   | yes                                                               | Unique identifier of the user                                                             |
| `email`         | `string`   | yes                                                               | E-Mail address of the user                                                                |
| `given_name`    | `string`   | yes                                                               | The given name (also known as first name) of the user                                     |
| `family_name`   | `string`   | yes                                                               | The family name (also know as last name) of the user                                      |
| `tenant_id`     | `string`   | if [tenant `assignment`](tenants.md) is `"by_external_tenant_id"` | Contains the identifier of the user's tenant                                              |
| `tariff_id`     | `string`   | if [tariffs](tariffs.md) are used                                 | The external id of the tariff. See [tariffs](tariffs.md) for further details              |
| `tariff_status` | `string`   | if [tariffs](tariffs.md) are used                                 | The external id of the tariff status. See [tariffs](tariffs.md) for further details       |
| `x_grp`         | `string[]` | no                                                                | A list of groups which the user is part of                                                |
| `phone_number`  | `string`   | no                                                                | The phone number of the user                                                              |
| `nickname`      | `string`   | no                                                                | Nickname of the user, typically used to prefill the display name of a meeting participant |

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

- `"opentalk-call-in"`: The service is allowed to provide a meeting [phone call-in service](call_in.md).
- `"opentalk-recorder"`: The service is allowed to provide a meeting [recording service](recorder.md).
