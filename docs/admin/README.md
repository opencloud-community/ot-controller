---
title: Controller
---

# Administration guide for the OpenTalk Controller

## General information about the service

- [Configuration](core/configuration.md)
- [HTTP Server](core/http_server.md) on which the controller offers its service
- [Migration guide for updating to new versions](migration/migration.md)
- [Command-line usage of the controller](cli/cli.md)
- [Configuration of multiple tenants](advanced/tenants.md)
- [Configuration of different tariffs and their capabilities](advanced/tariffs.md)
- [Execution of maintenance jobs](cli/jobs.md)
- [Modules that can be used in meetings](advanced/modules.md)
- [ACL management](advanced/acl.md)
- [Call-in](advanced/call_in.md)
- [Default and fallback values](advanced/defaults.md)
- [Endpoints](core/endpoints.md)
- [Logging](core/logging/log_output.md)
- [Metrics](core/logging/metrics.md)
- [STUN and TURN](core/stun_turn.md)
- [Personal Data Storage](personal_data_storage.md)

## Interaction between OpenTalk Controller and other services

### Services required by OpenTalk Controller

- [Database](core/database.md)
- [RabbitMQ](core/rabbitmq.md)
- [Redis](core/redis.md)
- [KeyCloak](core/keycloak.md)
- [MinIO](core/minio.md)

### Services that OpenTalk Controller can be integrated with

- [Shared folders on external systems](advanced/additional_services/shared_folder.md)
- [Tracing](core/logging/tracing.md)
- [Etherpad](advanced/additional_services/etherpad.md)
- [SpaceDeck](advanced/additional_services/spacedeck.md)

### Services that can interact with OpenTalk Controller

- [OpenTalk Obelisk](advanced/additional_services/obelisk.md) for handling dial-in from telephone line
- [OpenTalk Recorder](advanced/additional_services/recorder.md) for recording meetings
- [OpenTalk SMTP-Mailer](advanced/additional_services/smtp_mailer.md) for sending E-Mail notifications to users

## Under the hood

- [OIDC Authentication Flow](under_the_hood/oidc_auth.md)
- [Handling of WebAPI requests](under_the_hood/http_requests.md)
