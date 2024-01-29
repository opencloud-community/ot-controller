---
sidebar_position: 104
---

# Command-Line usage of `opentalk-controller`

When started without a subcommand, the `opentalk-controller` command loads the
[configuration](configuration.md) and starts as a service, serving incoming
connections using the included [HTTP server](http_server.md).

In addition, some subcommands are available for management tasks.

## Subcommands

These subcommands are available:

- [`fix-acl`](acl.md#opentalk-controller-fix-acl-subcommand) for recreating all ACL entries of the database.
- [`acl`](acl.md#opentalk-controller-acl-subcommand) for modification of ACL settings.
- [`migrate-db`](database.md#opentalk-controller-migrate-db-subcommand) for explicit migration of database without starting the controller service
- [`tenants`](tenants.md#opentalk-controller-tenants-subcommand) for managing tenants.
- [`tariffs`](tariffs.md#opentalk-controller-tariffs-subcommand) for managing tariffs.
- [`jobs`](jobs.md#opentalk-controller-jobs-subcommand) for configuring and running maintenance jobs.
- [`modules`](modules.md#opentalk-controller-modules-subcommand) for managing modules.
- `help` for showing the help output.

## Raw help output

Help output looks like this:

<!-- begin:fromfile:text:cli-usage/opentalk-controller-help -->

```text
Usage: opentalk-controller [OPTIONS] [COMMAND]

Commands:
  fix-acl     Recreate all ACL entries from the current database content. Existing entries will not be touched unless the command is told to delete them all beforehand
  acl         Modify the ACLs
  migrate-db  Migrate the db. This is done automatically during start of the controller, but can be done without starting the controller using this command
  tenants     Manage existing tenants
  tariffs     Manage tariffs
  jobs        Manage and execute maintenance jobs
  modules     Manage modules
  help        Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  Specify path to configuration file [default: config.toml]
      --reload           Triggers a reload of the Janus Server configuration
      --version
  -h, --help             Print help
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-help -->
