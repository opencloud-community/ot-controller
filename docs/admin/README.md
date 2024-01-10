---
title: Controller
---

# Administration guide for the OpenTalk Controller

## General information about the service

- [Configuration](configuration.md)
- [HTTP Server](http_server.md) on which the controller offers its service
- [Migration guide for updating to new versions](migration.md)
- [Command-line usage of the controller](cli.md)
- [Configuration of multiple tenants](tenants.md)
- [Configuration of different tariffs and their capabilities](tariffs.md)
- [Execution of maintenance jobs](jobs.md)
- [Modules that can be used in meetings](modules.md)
- [ACL management](acl.md)
- [Call-in](call_in.md)
- [Default and fallback values](defaults.md)
- [Endpoints](endpoints.md)
- [Logging](logging.md)
- [Metrics](metrics.md)
- [STUN and TURN](stun_turn.md)
- [Personal Data Storage](personal_data_storage.md)

## Interaction between OpenTalk Controller and other services

### Services required by OpenTalk Controller

- [Database](database.md)
- [RabbitMQ](rabbitmq.md)
- [Redis](redis.md)
- [KeyCloak](keycloak.md)
- [MinIO](minio.md)

### Services that OpenTalk Controller can be integrated with

- [Shared folders on external systems](shared_folder.md)
- [Tracing](tracing.md)
- [Etherpad](etherpad.md)
- [SpaceDeck](spacedeck.md)

### Services that can interact with OpenTalk Controller

- [OpenTalk Obelisk](obelisk.md) for handling dial-in from telephone line
- [OpenTalk Recorder](recorder.md) for recording meetings
- [OpenTalk SMTP-Mailer](smtp_mailer.md) for sending E-Mail notifications to users
