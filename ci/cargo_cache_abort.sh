#!/bin/bash

# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Script for canceling the last build step of the given commands, which is executed again in the corresponding jobs in order to reduce the CI time

commands=(
    "cargo install diesel_cli --version ~2 --no-default-features --features=\"postgres\""
    "cargo build --package opentalk-controller"
    "cargo build --release --locked"
    "cargo xtask --help"
    "cargo clippy --workspace --all-features --tests -- --deny warnings"
    "cargo auditable build --release --locked --workspace"
)

execute_and_check_for_abort_condition() {
    local cmd=$1
    echo "Executing: $cmd"

    eval "$cmd" 2>&1 | while read -r line; do

        if [[ $line == *"Compiling opentalk-controller "* ]]; then
            echo "Output contains 'Compiling opentalk-controller', continue with next command."
            return  1 # Return a non-zero status to indicate the command was cancelled
        fi

        echo "$line"
    done
    return  0
}

for cmd in "${commands[@]}"; do
    execute_and_check_for_abort_condition "$cmd"
    # Check the return status of the function
    if [ $? -eq  1 ]; then
        echo "Command was canceled, continue with next command."
    fi
done

echo "All commands attempted."
