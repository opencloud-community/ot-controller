#!/bin/bash

# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

# This script generates the mail worker protocol schema and example message files.

set -xe
set -o pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

DOCS_TEMP_DIR=target/docs/temporary

source "$SCRIPT_DIR"/include/codify.sh

export MAIL_WORKER_PROTOCOL_EXAMPLE_OUTPUT_DIR=$DOCS_TEMP_DIR/mail-worker-protocol

cargo run --package opentalk-mail-worker-protocol --bin export_schema_and_examples --features utoipa

for FILE in "$MAIL_WORKER_PROTOCOL_EXAMPLE_OUTPUT_DIR"/*.json; do
  codify json < "$FILE" > "$FILE.md"
done
