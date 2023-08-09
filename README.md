<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# OpenTalk Controller

See the [administration guide](docs/admin/README.md) for more information.

## Configuration

See the [configuration](docs/admin/configuration.md) chapter of the
administration guide for more information.

An example configuration is available in the [`extra/example.toml`](extra/
example.toml) file. It can be copied to the root directory:

```sh
cp ./extra/example.toml ./config.toml
```

## Upgrading

See the [migration guide](docs/admin/migration.md) for information about upgrading.

## Build the container image

The `Dockerfile` is located at `container/Dockerfile`.

To build the image, execute in the root of the repository:

```bash
docker build -f container/Dockerfile . --tag <your tag>
```

## Sub-crates

Inside the crates folder following crates can be found:

- [controller](crates/controller)
    - core crate which contains all of the controllers core features
    - OpenID Connect user authentication
    - Database connection and interfacing
    - `actix_web` based HTTP servers for external and internal APIs
    - Extensible signaling websocket endpoint for video room signaling
- [controller-settings](crates/controller-settings)
    - Settings for the controller
- [db-storage](crates/db-storage)
    - Database types used for the controller and modules
- [janus-media](crates/janus-media)
    - media signaling module using the `janus-client` crate
- [jobs](crates/jobs)
    - Job execution system for maintenance tasks such as removing old meeting information according to GDPR legislation
- [community-modules](crates/community-modules)
    - functionality for registering all modules in the community edition
    - depends on all modules in the community edition
- [chat](crates/chat)
    - chat signaling module which implements a simple room, group and private chat
- [automod](crates/automod)
    - signaling module implementing automoderation for videoconferences
- [legal-vote](crates/legal-vote)
    - signaling module implementing legal vote for videoconferences
- [polls](crates/polls)
    - signaling module implementing polls for videoconferences
- [client](crates/client) EXPERIMENTAL
    - Client side implementation of the controllers APIs used for testing
- [r3dlock](crates/r3dlock)
    - redis redlock distributed lock implementation for a single instance
- [janus-client](crates/janus-client)
    - janus client library for interfacing with the [Janus WebRTC Server](https://janus.conf.meetecho.com/)
- [kustos](crates/kustos)
    - authz abstraction based on casbin-rs
- [test-util](crates/test-util)
- [types](crates/types)
    - types that are shared across different crates, such as Web API and signaling messages
