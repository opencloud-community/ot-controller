# Tariffs

Tariffs in OpenTalk can be created, edited, and deleted with the commandline using `opentalk-controller tariffs <COMMAND>`.

A tariff can restrict the OpenTalk resource usage by imposing a combination of different quota types.
The following quotas are supported:

| Quota Name               | Description                                                                                                                             |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------- |
| `max_storage`            | The maximum allowed storage per user in bytes. This is a soft limit. When the limit is exceeded, a user can not store additional files. |
| `room_time_limit_secs`   | This quota restricts the total duration for which a tenant can utilize a meeting room, measured in seconds.                             |
| `room_participant_limit` | This quota sets a limit on the number of participants that can join a room.                                                             |

## `opentalk-controller tariffs` subcommand

This subcommand is used to manage tariffs.

Help output looks like this:

<!-- begin:fromfile:cli-usage/opentalk-controller-tariffs-help.md -->

```text
Manage tariffs

Usage: opentalk-controller tariffs <COMMAND>

Commands:
  list    List all available tariffs
  create  Create a new tariff
  delete  Delete a tariff by name
  edit    Modify an existing tariff
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

<!-- end:fromfile:cli-usage/opentalk-controller-tariffs-help.md -->

### Examples

#### List All Tariffs

Run `opentalk-controller tariffs list` to show all existing tariffs.

#### Create a New Tariff

Run `opentalk-controller tariffs create <TariffName> <ExternalTariffId>` to create a new tariff.

<!-- begin:fromfile:cli-usage/opentalk-controller-tariffs-create.md -->

```text
Create a new tariff

Usage: opentalk-controller tariffs create [OPTIONS] <TARIFF_NAME> <EXTERNAL_TARIFF_ID>

Arguments:
  <TARIFF_NAME>         Name of the tariff
  <EXTERNAL_TARIFF_ID>  Eternal ID to map to the tariff

Options:
      --disabled-modules <DISABLED_MODULES>    Comma-separated list of modules to disable
      --disabled-features <DISABLED_FEATURES>  Comma-separated list of features to disable
      --quotas <QUOTAS>                        Comma-separated list of key=value pairs
  -h, --help                                   Print help
```

<!-- end:fromfile:cli-usage/opentalk-controller-tariffs-create.md -->

#### Delete an Existing Tariff

Run `opentalk-controller tariffs delete <TariffName> <ExternalTariffId>` to delete an existing tariff.

<!-- begin:fromfile:cli-usage/opentalk-controller-tariffs-delete.md -->

```text
Delete a tariff by name

Usage: opentalk-controller tariffs delete <TARIFF_NAME>

Arguments:
  <TARIFF_NAME>  Name of the tariff to delete

Options:
  -h, --help  Print help
```

<!-- end:fromfile:cli-usage/opentalk-controller-tariffs-delete.md -->

#### Edit an Existing Tariff

Run `opentalk-controller tariffs edit <TariffName>` to edit an existing tariff.

Help output looks like this:

<!-- begin:fromfile:cli-usage/opentalk-controller-tariffs-edit.md -->

```text
Modify an existing tariff

Usage: opentalk-controller tariffs edit [OPTIONS] <TARIFF_NAME>

Arguments:
  <TARIFF_NAME>
          Name of the tariff to modify

Options:
      --set-name <SET_NAME>
          Set a new name

      --add-external-tariff-ids <ADD_EXTERNAL_TARIFF_IDS>
          Comma-separated list of external tariff_ids to add

      --remove-external-tariff-ids <REMOVE_EXTERNAL_TARIFF_IDS>
          Comma-separated list of external tariff_ids to remove

      --add-disabled-modules <ADD_DISABLED_MODULES>
          Comma-separated list of module names to add

      --remove-disabled-modules <REMOVE_DISABLED_MODULES>
          Comma-separated list of module names to remove

      --add-disabled-features <ADD_DISABLED_FEATURES>
          Comma-separated list of feature names to add

      --remove-disabled-features <REMOVE_DISABLED_FEATURES>
          Comma-separated list of feature names to remove

      --add-quotas <ADD_QUOTAS>
          Comma-separated list of key=value pairs to add, overwrites quotas with the same name

      --remove-quotas <REMOVE_QUOTAS>
          Comma-separated list of quota keys to remove

          Possible values:
          - max_storage:            This quota limits the total amount of data, measured bytes, that can be stored by the tenant. This is a soft limit which allows the user to store files as long as their usage is below the limit. Once the limit is reached or exceeded, no new data can be stored
          - room_time_limit_secs:   This quota restricts the total duration for which a tenant can utilize a meeting room, measured in seconds
          - room_participant_limit: This quota sets a limit on the number of participants that can join a room

  -h, --help
          Print help (see a summary with '-h')
```

<!-- end:fromfile:cli-usage/opentalk-controller-tariffs-edit.md -->

These subcommand options enable the modification of tariff names, external tariff IDs, disabled modules and features as well as quotas.
