#!/bin/bash

# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

set -xe

DOCS_TEMP_DIR=target/docs/temporary

OPENTALK_CONTROLLER_PROJECT=${OPENTALK_CONTROLLER_PROJECT:-opentalk-controller}
OPENTALK_CONTROLLER_CMD=${OPENTALK_CONTROLLER_CMD:-target/debug/opentalk-controller}

CLI_DIR="$DOCS_TEMP_DIR"/cli-usage
JOBS_DIR="$DOCS_TEMP_DIR"/jobs
CONFIG_DIR="$DOCS_TEMP_DIR"/config
CMDNAME=opentalk-controller

mkdir -p "$CLI_DIR" "$JOBS_DIR" "$CONFIG_DIR"

cp extra/example.toml "$CONFIG_DIR"/example.toml

cargo build --package "$OPENTALK_CONTROLLER_PROJECT"

$OPENTALK_CONTROLLER_CMD help > "$CLI_DIR"/"$CMDNAME"-help
$OPENTALK_CONTROLLER_CMD fix-acl --help > "$CLI_DIR"/"$CMDNAME"-fix-acl-help
$OPENTALK_CONTROLLER_CMD acl --help > "$CLI_DIR"/"$CMDNAME"-acl-help
$OPENTALK_CONTROLLER_CMD migrate-db --help > "$CLI_DIR"/"$CMDNAME"-migrate-db-help
$OPENTALK_CONTROLLER_CMD tenants --help > "$CLI_DIR"/"$CMDNAME"-tenants-help
$OPENTALK_CONTROLLER_CMD tenants list --help > "$CLI_DIR"/"$CMDNAME"-tenants-list-help
$OPENTALK_CONTROLLER_CMD tenants set-oidc-id --help > "$CLI_DIR"/"$CMDNAME"-tenants-set-oidc-id-help
$OPENTALK_CONTROLLER_CMD tariffs --help > "$CLI_DIR"/"$CMDNAME"-tariffs-help
$OPENTALK_CONTROLLER_CMD tariffs create --help > "$CLI_DIR"/"$CMDNAME"-tariffs-create
$OPENTALK_CONTROLLER_CMD tariffs delete --help > "$CLI_DIR"/"$CMDNAME"-tariffs-delete
$OPENTALK_CONTROLLER_CMD tariffs edit --help > "$CLI_DIR"/"$CMDNAME"-tariffs-edit
$OPENTALK_CONTROLLER_CMD jobs --help > "$CLI_DIR"/"$CMDNAME"-jobs-help
$OPENTALK_CONTROLLER_CMD jobs execute --help > "$CLI_DIR"/"$CMDNAME"-jobs-execute-help
$OPENTALK_CONTROLLER_CMD \
  --config extra/example.toml \
  jobs \
  execute \
  self-check \
  --hide-duration \
  > "$CLI_DIR"/"$CMDNAME"-jobs-execute-self-check
$OPENTALK_CONTROLLER_CMD jobs default-parameters --help > "$CLI_DIR"/"$CMDNAME"-jobs-default-parameters-help
$OPENTALK_CONTROLLER_CMD modules --help > "$CLI_DIR"/"$CMDNAME"-modules-help
$OPENTALK_CONTROLLER_CMD modules list --help > "$CLI_DIR"/"$CMDNAME"-modules-list-help

$OPENTALK_CONTROLLER_CMD --config extra/example.toml jobs default-parameters self-check > "$JOBS_DIR"/parameters-self-check.json
$OPENTALK_CONTROLLER_CMD --config extra/example.toml jobs default-parameters event-cleanup > "$JOBS_DIR"/parameters-event-cleanup.json
$OPENTALK_CONTROLLER_CMD --config extra/example.toml jobs default-parameters adhoc-event-cleanup > "$JOBS_DIR"/parameters-adhoc-event-cleanup.json
$OPENTALK_CONTROLLER_CMD --config extra/example.toml jobs default-parameters invite-cleanup > "$JOBS_DIR"/parameters-invite-cleanup.json

$OPENTALK_CONTROLLER_CMD --config extra/example.toml modules list > "$CLI_DIR"/"$CMDNAME"-modules-list

# Remove trailing spaces to prevent markdownlint from triggering *MD009 - Trailing spaces*
# https://github.com/markdownlint/markdownlint/blob/main/docs/RULES.md#md009---trailing-spaces
for file in "$CLI_DIR"/*; do
  sed --regexp-extended --in-place 's#[[:space:]]+$##g' "$file"
done

cargo run --bin ci-doc-updater -- generate --raw-files-dir target/docs/temporary/ --documentation-dir docs/
