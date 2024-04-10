#!/bin/bash

# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

set -xe

DOCS_TEMP_DIR=target/docs/temporary

OPENTALK_CONTROLLER_PROJECT=${OPENTALK_CONTROLLER_PROJECT:-opentalk-controller}
OPENTALK_CONTROLLER_CMD=${OPENTALK_CONTROLLER_CMD:-target/release/opentalk-controller}

DB_DIR="$DOCS_TEMP_DIR"/database
ER_DIAGRAM_MERMAID="$DB_DIR/er-diagram.mermaid"

mkdir -p "$DB_DIR"

$OPENTALK_CONTROLLER_CMD --config extra/example.toml migrate-db
sqlant -o mermaid $OPENTALK_CTRL_DATABASE__URL > $ER_DIAGRAM_MERMAID
