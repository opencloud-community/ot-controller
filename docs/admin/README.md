<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
SPDX-License-Identifier: EUPL-1.2
-->

# Administration guide for the OpenTalk Controller

## General information about the service

- [Configuration](configuration.md)
- [HTTP Server](http_server.md) on which the controller offers its service
- [Migration guide for updating to new versions](migration.md)
- [Command-line usage of the controller](cli.md)
- [Configuration of multiple tenants](tenants.md)
- [Configuration of different tariffs and their capabilities](tariffs.md)
- [Execution of maintenance jobs](jobs.md)

## Interaction between OpenTalk Controller and other sevrices

### Services required by OpenTalk Controller

- [Database](database.md)
- [RabbitMQ](rabbitmq.md)
- [Redis](redis.md)
- [KeyCloak](keycloak.md)
- [MinIO](minio.md)

### Services that OpenTalk Controller can be integrated with

- [Shared folders on external systems](shared_folder.md)
- [OpenTelemetry](opentelemetry.md)
- [Etherpad](etherpad.md)
- [SpaceDeck](spacedeck.md)

### Services that can interact with OpenTalk Controller

- [OpenTalk Obelisk](obelisk.md) for handling dial-in from telephone line
- [OpenTalk Recorder](recorder.md) for recording meetings
- [OpenTalk SMTP-Mailer](smtp_mailer.md) for sending E-Mail notifications to users
