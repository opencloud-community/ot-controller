# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

FROM rust:slim AS Builder

WORKDIR /builds/opentalk/backend/services/controller
ENV CARGO_HOME=/builds/opentalk/backend/services/controller/cargo

RUN apt-get update && apt-get install -y libpq-dev protobuf-compiler

RUN cargo build --release --locked

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y libpq5 ca-certificates

WORKDIR /builds/opentalk/backend/services/controller

COPY --from=Builder /builds/opentalk/backend/services/controller/target/release/opentalk-controller .
RUN ln -s opentalk-controller k3k-controller
COPY extra/example.toml config.toml

EXPOSE 11311
ENTRYPOINT [ "./opentalk-controller" ]
