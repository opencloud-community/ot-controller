#!/bin/bash

# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

set -xe

DOCS_TEMP_DIR=target/docs/temporary

CONTROLLER_CMD=target/debug/opentalk-controller

CLI_DIR="$DOCS_TEMP_DIR"/cli-usage
JOBS_DIR="$DOCS_TEMP_DIR"/jobs
CONFIG_DIR="$DOCS_TEMP_DIR"/config
CMDNAME=opentalk-controller

mkdir -p "$CLI_DIR" "$JOBS_DIR" "$CONFIG_DIR"

cp extra/example.toml "$CONFIG_DIR"/example.toml

cargo build --package opentalk-controller

$CONTROLLER_CMD help > "$CLI_DIR"/"$CMDNAME"-help
$CONTROLLER_CMD fix-acl --help > "$CLI_DIR"/"$CMDNAME"-fix-acl-help
$CONTROLLER_CMD acl --help > "$CLI_DIR"/"$CMDNAME"-acl-help
$CONTROLLER_CMD migrate-db --help > "$CLI_DIR"/"$CMDNAME"-migrate-db-help
$CONTROLLER_CMD tenants --help > "$CLI_DIR"/"$CMDNAME"-tenants-help
$CONTROLLER_CMD tariffs --help > "$CLI_DIR"/"$CMDNAME"-tariffs-help
$CONTROLLER_CMD jobs --help > "$CLI_DIR"/"$CMDNAME"-jobs-help
$CONTROLLER_CMD jobs execute --help > "$CLI_DIR"/"$CMDNAME"-jobs-execute-help
$CONTROLLER_CMD \
  --config extra/example.toml \
  jobs \
  execute \
  self-check \
  --hide-duration \
  > "$CLI_DIR"/"$CMDNAME"-jobs-execute-self-check
$CONTROLLER_CMD jobs default-parameters --help > "$CLI_DIR"/"$CMDNAME"-jobs-default-parameters-help

$CONTROLLER_CMD --config extra/example.toml jobs default-parameters self-check > "$JOBS_DIR"/parameters-self-check.json
$CONTROLLER_CMD --config extra/example.toml jobs default-parameters event-cleanup > "$JOBS_DIR"/parameters-event-cleanup.json

# Remove trailing spaces to prevent markdownlint from triggering *MD009 - Trailing spaces*
# https://github.com/markdownlint/markdownlint/blob/main/docs/RULES.md#md009---trailing-spaces
for file in "$CLI_DIR"/*; do
  sed --regexp-extended --in-place 's#[[:space:]]+$##g' "$file"
done

cargo run --bin ci-doc-updater -- generate --raw-files-dir target/docs/temporary/ --documentation-dir docs/
