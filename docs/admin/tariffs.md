---
sidebar_position: 106
title: Configuration of different tariffs and their capabilities
---

# Tariffs

Tariffs in OpenTalk can be created, edited, and deleted with the commandline using `opentalk-controller tariffs <COMMAND>`.

## `opentalk-controller tariffs` subcommand

This subcommand is used to manage tariffs.

Help output looks like this:

<!-- begin:fromfile:text:cli-usage/opentalk-controller-tariffs-help -->

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

<!-- end:fromfile:text:cli-usage/opentalk-controller-tariffs-help -->

### Examples

#### List All Tariffs

Run `opentalk-controller tariffs list` to show all existing tariffs.

#### Create a New Tariff

Run `opentalk-controller tariffs create <TariffName> <ExternalTariffId>` to create a new tariff.

<!-- begin:fromfile:text:cli-usage/opentalk-controller-tariffs-create -->

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

<!-- end:fromfile:text:cli-usage/opentalk-controller-tariffs-create -->

#### Delete an Existing Tariff

Run `opentalk-controller tariffs delete <TariffName> <ExternalTariffId>` to delete an existing tariff.

<!-- begin:fromfile:text:cli-usage/opentalk-controller-tariffs-delete -->

```text
Delete a tariff by name

Usage: opentalk-controller tariffs delete <TARIFF_NAME>

Arguments:
  <TARIFF_NAME>  Name of the tariff to delete

Options:
  -h, --help  Print help
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-tariffs-delete -->

#### Edit an Existing Tariff

Run `opentalk-controller tariffs edit <TariffName>` to edit an existing tariff.

Help output looks like this:

<!-- begin:fromfile:text:cli-usage/opentalk-controller-tariffs-edit -->

```text
Modify an existing tariff

Usage: opentalk-controller tariffs edit [OPTIONS] <TARIFF_NAME>

Arguments:
  <TARIFF_NAME>  Name of the tariff to modify

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
  -h, --help
          Print help
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-tariffs-edit -->

These subcommand options enable the modification of tariff names, external tariff IDs, disabled modules and features as well as quotas.
