---
sidebar_position: 105
title: Configuration of multiple tenants
---

# Tenants

A deployment of OpenTalk is capable of handling multiple completely separated
organizations, named *tenant*. Inside such a system, each tenant is handled the
same way as if they had a separate deployment. Some exceptions to that rule
exist though, e.g. all login information for OpenTalk is supplied by the same
identity provider.

By default, OpenTalk is configured that only a single, automatically created,
tenant named `OpenTalkDefaultTenant` exists. Therefore instances where tenants
don't matter need no extra configuration.

## Configuration

The section in the [configuration file](configuration.md) is called `tenants`.

| Field                                    | Type     | Required | Default value             | Description                                                                        |
| ---------------------------------------- | -------- | -------- | ------------------------- | ---------------------------------------------------------------------------------- |
| `assignment`                             | `string` | no       | `"static"`                | The method used to assign tenants. Either `"static"` or `"by_external_tenant_id"`. |
| `static_tenant_id`                       | `string` | no       | `"OpenTalkDefaultTenant"` | The tenant id used in the database when `assignment` is `"static"`.                |
| `external_tenant_id_user_attribute_name` | `string` | no       | `"tenant_id"`             | The attribute by which external user search is restricted to a tenant.             |

Setting `assignment` to `"static"` (which is the default value), disables
tenancy in the deployment. From a technical view, this is implemented by having
one default tenant containing all users. This tenant exists in the database and
is named by the value of `static_tenant_id` with `"OpenTalkDefaultTenant"` being
the default value.

In order to use multiple tenants, `assignment` must be set to
`"by_external_tenant_id"`. This requires the `tenant_id` field
in the [authentication information sent by KeyCloak](keycloak.md#jwt-fields-for-user-login).
Whenever a user logs in with a `tenant_id` that is unknown to the OpenTalk
controller, a new entry for this tenant is created in the database. Because of
that, the command-line tooling provides no option for adding tenants to the
database.

If the [find endpoint allows searching users on the KeyCloak](endpoints.md),
then the results found on the KeyCloak will be filtered by the tenant of the
currently logged-in user. The KeyCloak attribute used for filtering is defined
by the value of `external_tenant_id_user_attribute_name` which defaults to
`"tenant_id"`. :warning: Beware that this only affects the search which is
performed through the KeyCloak Web API, so that the name of the **KeyCloak
attribute** is not enforced there, in contrast to the **JWT claim** which must
always be configured as `tenant_id`.

### Example configurations

#### Configuration for using a static tenant assignment

This is the default configuration that is applied by OpenTalk when no
`[tenants]` section exists in the configuration file.

```toml
[tenants]
assignment = "static"
static_tenant_id = "OpenTalkDefaultTenant"
```

#### Configuration for using tenant functionality

```toml
[tenants]
assignment = "by_external_tenant_id"
external_tenant_id_user_attribute_name = "tenant_id"
```

## `opentalk-controller tenants` subcommand

This subcommand is used to manage tenants.

Help output looks like this:

<!-- begin:fromfile:text:cli-usage/opentalk-controller-tenants-help -->

```text
Manage existing tenants

Usage: opentalk-controller tenants <COMMAND>

Commands:
  list         List all available tenants
  set-oidc-id  Change a tenants oidc-id
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-tenants-help -->

## `opentalk-controller tenants list` subcommand

<!-- begin:fromfile:text:cli-usage/opentalk-controller-tenants-list-help -->

```text
List all available tenants

Usage: opentalk-controller tenants list

Options:
  -h, --help  Print help
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-tenants-list-help -->

## `opentalk-controller tenants set-oidc-id` subcommand

<!-- begin:fromfile:text:cli-usage/opentalk-controller-tenants-set-oidc-id-help -->

```text
Change a tenants oidc-id

Usage: opentalk-controller tenants set-oidc-id <ID> <NEW_OIDC_ID>

Arguments:
  <ID>
  <NEW_OIDC_ID>

Options:
  -h, --help  Print help
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-tenants-set-oidc-id-help -->
