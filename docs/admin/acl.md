<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
SPDX-License-Identifier: EUPL-1.2
-->

# ACL management

<!-- TODO -->
:construction: This section has not been finished yet.

## `opentalk-controller acl` subcommand

<!-- TODO -->
:construction: This section has not been finished yet.

This subcommand is used modify ACLs.

### Help output

Help output looks like this:

<!-- begin:fromfile:text:cli-usage/opentalk-controller-acl-help -->

```text
Modify the ACLs

Usage: opentalk-controller acl <COMMAND>

Commands:
  users-have-access-to-all-rooms  Allows all users access to all rooms
  help                            Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-acl-help -->

## `opentalk-controller fix-acl` subcommand

<!-- TODO -->
:construction: This section has not been finished yet.

This subcommand is used to recreate all ACL entries from the current database content.

### Help output

Help output looks like this:

<!-- begin:fromfile:text:cli-usage/opentalk-controller-fix-acl-help -->

```text
Recreate all ACL entries from the current database content. Existing entries will not be touched unless the command is told to delete them all beforehand

Usage: opentalk-controller fix-acl [OPTIONS]

Options:
      --delete-acl-entries
          !DANGER! Removes all ACL entries before running any fixes.

          Requires all fixes to be run.

      --skip-users
          Skip user role fix

      --skip-groups
          Skip group membership fix

      --skip-rooms
          Skip fix of room permissions

      --skip-module-resources
          Skip fix of module resources permissions

      --skip-events
          Skip fix of event permission fixes

  -h, --help
          Print help (see a summary with '-h')
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-fix-acl-help -->
