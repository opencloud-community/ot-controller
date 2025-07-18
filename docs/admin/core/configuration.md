---
title: Configuration
---

# Configuring OpenTalk-Controller

When the controller gets started, it loads the configuration from the
environment. It reads the settings in this order:

- Read environment variables which have a specific name, see section
  [Environment variables](#environment-variables).
- Load from a configuration file which defaults to `config.toml` in the current
  working directory, but can be set using the `--config` or `-c` CLI argument

## Sections in the configuration file

Functionality that can be configured through the configuration file:

- [Authz](../advanced/acl.md)
- [Call-in](../advanced/call_in.md)
- [Database](database.md)
- [Default and fallback values](../advanced/defaults.md)
- [Endpoints](endpoints.md)
- [EtherPad](../advanced/additional_services/etherpad.md)
- [HTTP server](http_server.md)
- [Logging](logging/log_output.md)
- [Metrics](logging/metrics.md)
- [MinIO](minio.md)
- [Monitoring](monitoring.md)
- [OIDC Identity Provider](./keycloak.md)
- [RabbitMQ](rabbitmq.md)
    - The recording service is enabled/disabled by configuring the queue name
- [Redis](redis.md)
- [Room server](room_server.md)
- [Shared folders on external storage systems](../advanced/additional_services/shared_folder.md)
- [SpaceDeck](../advanced/additional_services/spacedeck.md)
- [Subroom Audio](subroom_audio.md)
- [Tariffs](../advanced/tariffs.md)
- [Tenants](../advanced/tenants.md)
- [User search](./user_search.md)

## Environment variables

Settings in the configuration file can be overwritten by environment variables,
nested fields are separated by two underscores `__`. The pattern looks like
this:

```sh
OPENTALK_CTRL_<field>__<nested-field>…
```

### Limitations

Some settings can not be overwritten by environment variables. This is for
example the case for entries in lists, because there is no environment variable
naming pattern that could identify the index of the entry inside the list.

### Examples

In order to set the `database.url` field, this environment variable could be used:

```sh
OPENTALK_CTRL_DATABASE__URL=postgres://opentalk:s3cur3_p4ssw0rd@localhost:5432/opentalk
```

The field `database.max_connections` could be overwritten like this:

```sh
OPENTALK_CTRL_DATABASE__MAX_CONNECTIONS=5
```

The field `tariffs.status_mapping.downgraded_tariff_name` could be overwritten like this:

```sh
OPENTALK_CTRL_TARIFFS__STATUS_MAPPING__DOWNGRADED_TARIFF_NAME=downgraded_tariff
```

## Example configuration file

This file can be found in the source code distribution under `example/controller.toml`

<!-- begin:fromfile:config/controller.toml.md -->

```toml
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

[frontend]
# The base URL of the frontend.
base_url = "https://example.com"

# Information regarding the operator that is responsible for user-facing communication and legal
# disclosure.
#[operator_information]
# The URL where users can find the data protection or privacy policy.
#data_protection_url = "https://example.com/data-protection"
# The phone number users can call for support.
#support_phone_number = "+493012345678"
# The email address users can contact for support.
#support_email_address = "support@example.com"

[logging]
# Default tracing directives that will always be applied after RUST_LOG's directives.
# Each array entry may contain a single directive.
# Below are some example directives which are used by default to reduce the amount of spamming some crates do by default.
# The defaults will always apply, but they can be explicitly overwritten with this config or the RUST_LOG environment
# variable. The priority of the different configuration options is: RUST_LOG > config file > defaults
#default_directives = [
#   "ERROR",
#   "opentalk=INFO",
#   "pinky_swear=OFF",
#   "rustls=WARN",
#   "mio=ERROR",
#   "lapin=WARN",
#   "execution_id=trace"
#]

# Specify an optional OTLP tracing endpoint to export traces to
#otlp_tracing_endpoint = "http://localhost:4317"

# Service name when using opentelemetry
#service_name = "controller"
# Service namespace when using opentelemetry
#service_namespace = "opentalk"
# Service instance id when using opentelemetry. A random UUID will be generated at runtime if not set here.
#service_instance_id = "627cc493-f310-47de-96bd-71410b7dec09"

[database]
# URL used to connect to a postgres.
url = "postgres://postgres:password123@localhost:5432/opentalk"

# Maximum number of connections allowed to the server.
# Defaults to 100 which is the default of postgres.
#max_connections = 100

#[monitoring]
#addr = "0.0.0.0"
#port = 11411

#[http]
# An optional address to which to bind.
# Can be either a hostname, or an IP address.
#
# By default, this will accept requests on both the IPv4 and IPv6 interfaces on any address.
#
# The exception to this rule is "::0" which will bind to both the IPv4 and the
# IPv6 UNSPECIFIED address, accepting requests on both of them. If the operating
# system provides no IPv6 support, or the service should not bind to an IPv6
# interface, "0.0.0.0" can be used instead, which will only bind to the IPv4
# UNSPECIFIED address.
#addr = "::0"
#
# A hostname or fully qualified domain name will bind to whatever the name
# resolution returns, either one or both IP protocols.
#addr = "controller.opentalk.example.com"
#addr = "localhost"
#addr = "opentalkserver"
#
# An explicit IPv4 or IPv6 address, will bind to the corresponding IP protocol.
#addr = "127.0.0.1"
#addr = "::1"
#addr = "192.0.2.0"
#addr = "2001:0DB8::1337:DEAD:CAFE"

# The port to bind the HTTP Server to (defaults to 11311).
#port = 11311

# Settings for the Keycloak which is the user provider and allows authentication via OIDC.
# This is deprecated, replace with [oidc] and [user_search] sections.
#[keycloak]
# URL to the Keycloak
#base_url = "http://example.com/auth"
# Name of the Keycloak realm
#realm = "MyRealm"
# Client ID
#client_id = "Controller"
# Client secret (application requires confidential client).
#client_secret = "v3rys3cr3t"

# Configure how Keycloak users resulting from a search and registered Opentalk users are assigned to each other
# The following assignment strategies are available:
#   - by Keycloak id (default): Keycloak users are assigned to Opentalk users using Keycloak's id field.
#   - by user attribute: Keycloak must provide a user attribute holding the user IDs. The name of this user
#                        attribute must be set here in external_id_user_attribute_name.
#external_id_user_attribute_name = "my_user_attribute_name"

# OIDC configuration.
# Currently only Keycloak is supported. Full compliance with other OIDC providers is not guaranteed.
[oidc]
# Base url for the OIDC authority. Will be used for frontend and controller unless overwritten by
# `oidc.frontend.authority` or `oidc.controller.authority`.
authority = "https://example.com/auth/realms/OPENTALK"

[oidc.frontend]
# OIDC authority base url for the frontend.
# Optional, if not set, the value from `oidc.authority` is used.
# Will be made available to the frontend via the `GET /v1/auth/login` endpoint.
#authority = "https://example.com/auth/realms/OPENTALK"

# Client id that will be used by the frontend when connecting to the oidc provider.
client_id = "Frontend"

[oidc.controller]
# OIDC authority base url for the controller.
# Optional, if not set, the value from `oidc.authority` is used.
#authority = "https://example.com/auth/realms/OPENTALK"

# Client id that will be used by the controller when connecting to the oidc provider.
client_id = "Controller"

# Client secret that will be used by the controller when connecting to the oidc provider.
client_secret = "v3rys3cr3t"

[user_search]
# Defines which backend to use for user search. Only `keycloak_webapi` is currently available.
backend = "keycloak_webapi"

# Base URL of the Keycloak web api.
api_base_url = "https://example.com/auth/admin/realms/OPENTALK"

# Client id that is used to authenticate against the user search API.
# Optional, if not set, the value from `oidc.controller.client_id` is used.
client_id = "Controller"

# Client secret that is used to authenticate against the user search API.
# Optional, if not set, the value from `oidc.controller.client_secret` is used.
client_secret = "v3rys3cr3t"

# Configure how Keycloak users resulting from a search and registered Opentalk users are assigned to each other
# The following assignment strategies are available:
#   - by Keycloak id (default): Keycloak users are assigned to Opentalk users using Keycloak's id field.
#   - by user attribute: Keycloak must provide a user attribute holding the user IDs. The name of this user
#                        attribute must be set here in external_id_user_attribute_name.
#external_id_user_attribute_name = "my_user_attribute_name"

# Set the behavior of the `/users/find` endpoint.
# This allows searching for users who have not yet logged into the controller.
# You can choose where to search for users or disable the endpoint completely for performance or privacy reasons.
# Possible values are "disabled", "from_database" and "from_user_search_backend".
users_find_behavior = "from_user_search_backend"

# LiveKit WebRTC SFU
[livekit]
public_url = "wss://url.to.your.livekit.server"
service_url = "https://localhost:7880"

api_key = "your-livekit-api-key"
api_secret = "your-livekit-api-secret"

[rabbit_mq]
# The URL to use to connect to the rabbit mq broker
#url = "amqp://guest:guest@localhost:5672"

# The rabbitmq queue name for the mail worker,
# mailing is disabled when this is not set.
#mail_task_queue = "opentalk_mailer"

# The rabbitmq queue name for the recorder,
# recording is disabled when this is not set.
#recording_task_queue = "opentalk_recorder"

# Minimum amount of connections to retain when removing stale connections
#min_connections = 10

# Maximum number of amqp channels per connection
#max_channels_per_connection = 100

#[redis]
# Configuration of a redis server which can be used for synchronizing multiple
# controllers running in a cluster to provide an OpenTalk web api and meeting
# signaling service.
#
# If this section is present, then the redis service will be used for
# synchronizing meeting state throughout all controllers in the cluster. If it
# is left out entirely, then the service will run in "standalone" mode.
#
# Redis URL used to connect the redis server
#url = "redis://localhost:6379/"

#[roomserver]
# Configure a roomserver for this controller.
#
# When enabled, the controllers built in signaling endpoints (`rooms/<room-id>/start` & `rooms/<room-id>/start_invited`)
# are be disabled and roomserver signaling endpoints (`rooms/roomserver/<room-id>/start` &
# `rooms/roomserver/<room-id>/start_invited`) have to be used instead.
#
# The deployed frontend client has to be compatible with the roomservers signaling implementation.
#
# The URL of the roomserver. Needs to be reachable by clients
#url = "http://localhost:11333"
# The roomservers API key
#api_token = "secret"

#[authz]
# Should the controller publish/receive ACL changes via RabbitMQ to/from other controllers
#synchronize_controllers = true

#[call_in]
# Set a phone number which will be displayed to the user
# for the call-in service
#tel="+493012345678"
# Enable the mapping of user names to their phone number. This requires
# the OIDC provider to have a phone number field configured for their users.
#enable_phone_mapping=false
# The default country code for call in numbers. Notated in Alpha-2 code (ISO 3166)
# Phone numbers that do not fall in the category of the default country must be notated
# in the international format.
#default_country_code="DE"

# MinIO configuration
[minio]
# The URI to the MinIO instance
uri = "http://localhost:9555"
# Name of the bucket
bucket = "controller"
# Access key for the MinIO bucket
access_key = "minioadmin"
# Secret key for the MinIO bucket
secret_key = "minioadmin"

# Etcd configuration
[etcd]
# A list urls of a etcd cluster
urls = ["localhost:2379"]

# The etherpad configuration for the meeting-notes module
#[etherpad]
#url = "http://localhost:9001"
# Etherpads api key
#api_key = "secret"

# Spacedeck configuration
#[spacedeck]
#url = "http://localhost:9666"
#api_key = "secret"

# Subroom audio whisper configuration
#[subroom_audio]
#enable_whisper = false

# Shared folder configuration
#[shared_folder]
#provider = "nextcloud"
#url = "https://nextcloud.example.org/"
#username = "exampleuser"
#password = "v3rys3cr3t"
# Optional subdirectory under the user's folder
#directory = "opentalk/meetings"
# Optional expiry of folder shares in days
#expiry = 48

# Default/fallback values
#[defaults]
# Default language of a new user
#user_language = "en-US"
# Default presenter role for all users (defaults to false if not set)
#screen_share_requires_permission = false
# A list of disabled features in the controller. By default all features are enabled.
# Format: <module>::<feature>. A missing module defaults to "core".
# Currently supported features:
# - core::call_in
# - integration::outlook
#disabled_features = ["core::call_in", "integration::outlook"]

# Settings for endpoints
#[endpoints]
# Disable the /users/find endpoint for performance or privacy reasons.
# This is deprecated, replace with `user_search.users_find_behavior`.
#disable_users_find = false

# Enable user-searching using Keycloak's admin API.
# This allows for finding users which have not yet logged into the controller
# This is deprecated, replace with `user_search.users_find_behavior`.
#users_find_use_kc = false

# Allow inviting any unchecked email address.
# Not recommended without proper outgoing anti-spam protection
#event_invite_external_email_address = false

# Prohibit users from changing the display name (guests are always allowed to change it)
#disallow_custom_display_name = false

# Disable the OpenAPI endpoint under `/v1/openapi.json` and the corresponding
# swagger endpoint under `/swagger`.
#disable_openapi = false

# Configuration for the /metrics HTTP endpoint
#[metrics]
# Allowlist for the /metrics endpoint
#
# Example: Allow all traffic from localhost
#allowlist = ["127.0.0.0/24", "::ffff:0:0/96"]

#[tenants]
# Configure how users are assigned to tenants
# The following assignment strategies are available:
#   - "static" (default): Every user is assigned to a single tenant with a tenant_id specified in the static_tenant_id field.
#                         static_tenant_id's default value is "OpenTalkDefaultTenant".
#   - "by_external_tenant_id": The OIDC provider (Keycloak) must be configured to include a "tenant_id" field in its
#                              id_token's JWT claims. It is used to assign users to the correct tenant.
#                              The OIDC provider must also provide a user attribute holding the tenant_id. The name of this
#                              field ist set here in external_tenant_id_user_attribute_name, the default field name is "tenant_id".
#
# Static assignment (Default configuration if nothing is specified):
#assignment = "static"
#static_tenant_id = "OpenTalkDefaultTenant"
#
# Assignment by JWT tenant_id:
#assignment = "by_external_tenant_id"
#external_tenant_id_user_attribute_name = "tenant_id"

#[tariffs]
# Configure how tariffs are assigned to users
# The following assignment strategies are available:
#   - "static" (default): Every user is assigned the same tariff with the tariff's name specified in a separate field
#               called static_tariff_name. The default value is "OpenTalkDefaultTariff".
#   - "by_external_tariff_id": The OIDC provider (Keycloak) must be configured to include a "tariff_id" field it's
#                              id_token's JWT claims. It is used to assign users the correct tariff.
#
# Static Example (Default configuration if nothing is specified)
#assignment = "static"
#static_tariff_name = "OpenTalkDefaultTariff"
#
# Assignment by JWT tariff_id example:
#assignment = "by_external_tariff_id"
#
# Status mapping for tariff status. Can only be used if the tariff assignment
# is configured as "by_external_tariff_id". If present, the controller will look
# at the JWT attribute named "tariff_status" and transfer its value to its
# internal tariff status based on the values of the "default", "paid" and
# "downgraded" field values. An entry in any of the lists below must be unique
# across all lists. For example, if the paid list contains "all_ok", then
# "all_ok" must not appear in any other list.
#[tariffs.status_mapping]
#
# The name of the tariff that gets applied when the user's tariff state is "downgraded".
#downgraded_tariff_name = "basic"
#
# The default tariff status, usually a tariff that any user can get without paying
# Any user with an invalid value in the "tariff_status" attribute will be set to
# the default status, but a warning will be issued if the mapping does not contain
# that attribute value.
#default = ["default"]
#
# The user's tariff has been paid and is valid.
#paid = ["paid", "all_ok"]
#
# The user has booked a specific tariff, but is not allowed to use it, e.g. because
# it is unpaid. Therefore the user's tariff is downgraded to the fallback tariff.
#downgraded = ["unpaid"]
```

<!-- end:fromfile:config/controller.toml.md -->
