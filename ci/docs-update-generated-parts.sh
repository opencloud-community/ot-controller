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

codify() {
  if [ -z "$1" ]; then
    echo "Error: no language specified"
    return 1
  fi

  echo "\`\`\`$1"
  while IFS= read -r data; do
    echo "$data"
  done
  echo '```'
}

mkdir -p "$CLI_DIR" "$JOBS_DIR" "$CONFIG_DIR"

cargo build --package "$OPENTALK_CONTROLLER_PROJECT"

codify toml < extra/example.toml > "$CONFIG_DIR"/example.toml.md

$OPENTALK_CONTROLLER_CMD help | codify text > "$CLI_DIR"/"$CMDNAME"-help.md
$OPENTALK_CONTROLLER_CMD fix-acl --help | codify text > "$CLI_DIR"/"$CMDNAME"-fix-acl-help.md
$OPENTALK_CONTROLLER_CMD acl --help | codify text > "$CLI_DIR"/"$CMDNAME"-acl-help.md
$OPENTALK_CONTROLLER_CMD migrate-db --help | codify text > "$CLI_DIR"/"$CMDNAME"-migrate-db-help.md
$OPENTALK_CONTROLLER_CMD tenants --help | codify text > "$CLI_DIR"/"$CMDNAME"-tenants-help.md
$OPENTALK_CONTROLLER_CMD tenants list --help | codify text > "$CLI_DIR"/"$CMDNAME"-tenants-list-help.md
$OPENTALK_CONTROLLER_CMD tenants set-oidc-id --help | codify text > "$CLI_DIR"/"$CMDNAME"-tenants-set-oidc-id-help.md
$OPENTALK_CONTROLLER_CMD tariffs --help | codify text > "$CLI_DIR"/"$CMDNAME"-tariffs-help.md
$OPENTALK_CONTROLLER_CMD tariffs create --help | codify text > "$CLI_DIR"/"$CMDNAME"-tariffs-create.md
$OPENTALK_CONTROLLER_CMD tariffs delete --help | codify text > "$CLI_DIR"/"$CMDNAME"-tariffs-delete.md
$OPENTALK_CONTROLLER_CMD tariffs edit --help | codify text > "$CLI_DIR"/"$CMDNAME"-tariffs-edit.md
$OPENTALK_CONTROLLER_CMD jobs --help | codify text > "$CLI_DIR"/"$CMDNAME"-jobs-help.md
$OPENTALK_CONTROLLER_CMD jobs execute --help | codify text > "$CLI_DIR"/"$CMDNAME"-jobs-execute-help.md
$OPENTALK_CONTROLLER_CMD \
  --config extra/example.toml \
  jobs \
  execute \
  self-check \
  --hide-duration \
  | codify text > "$CLI_DIR"/"$CMDNAME"-jobs-execute-self-check.md
$OPENTALK_CONTROLLER_CMD jobs default-parameters --help | codify text > "$CLI_DIR"/"$CMDNAME"-jobs-default-parameters-help.md
$OPENTALK_CONTROLLER_CMD modules --help | codify text > "$CLI_DIR"/"$CMDNAME"-modules-help.md
$OPENTALK_CONTROLLER_CMD modules list --help | codify text > "$CLI_DIR"/"$CMDNAME"-modules-list-help.md

$OPENTALK_CONTROLLER_CMD --config extra/example.toml jobs default-parameters self-check | codify json > "$JOBS_DIR"/parameters-self-check.json.md
$OPENTALK_CONTROLLER_CMD --config extra/example.toml jobs default-parameters event-cleanup | codify json > "$JOBS_DIR"/parameters-event-cleanup.json.md
$OPENTALK_CONTROLLER_CMD --config extra/example.toml jobs default-parameters adhoc-event-cleanup | codify json > "$JOBS_DIR"/parameters-adhoc-event-cleanup.json.md
$OPENTALK_CONTROLLER_CMD --config extra/example.toml jobs default-parameters invite-cleanup | codify json > "$JOBS_DIR"/parameters-invite-cleanup.json.md

$OPENTALK_CONTROLLER_CMD --config extra/example.toml modules list | codify text > "$CLI_DIR"/"$CMDNAME"-modules-list.md

# Remove trailing spaces to prevent markdownlint from triggering *MD009 - Trailing spaces*
# https://github.com/markdownlint/markdownlint/blob/main/docs/RULES.md#md009---trailing-spaces
for file in "$CLI_DIR"/*; do
 # Check if the script is running on macOS or BSD
 if [[ "$(uname)" == "Darwin" ]] || [[ "$(uname)" == "BSD" ]]; then
    sed -i '' -E 's/[[:space:]]+$//' "$file"
 else
    # For other Linux/Unix-like systems
    sed --in-place --regexp-extended 's/[[:space:]]+$//' "$file"
 fi
done

cargo run --bin ci-doc-updater -- generate --raw-files-dir target/docs/temporary/ --documentation-dir docs/
