---
sidebar_position: 106
title: Configuration of different streaming services and their capabilities
---

# Streaming Services

Streaming services in OpenTalk can be created, edited, and deleted with the commandline using `opentalk-controller streaming-services <COMMAND>`.

## `opentalk-controller streaming-services` subcommand

This subcommand is used to manage streaming services.

Help output looks like this:

<!-- begin:fromfile:text:cli-usage/opentalk-controller-streaming-services-help -->

```text
Manage streaming services

Usage: opentalk-controller streaming-services <COMMAND>

Commands:
  list    List all available streaming services
  create  Create a new streaming service
  delete  Delete a streaming service by id
  edit    Modify an existing streaming service
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-streaming-services-help -->

### Examples

#### List All Streaming Services

Run `opentalk-controller streaming-services list` to show all existing streaming services.

#### Create a New Streaming Service

Run `opentalk-controller streaming-services create <StreamingServiceName> <Kind>` to create a new streaming service.

<!-- begin:fromfile:text:cli-usage/opentalk-controller-streaming-services-create -->

```text
Create a new streaming service

Usage: opentalk-controller streaming-services create <NAME> <KIND> [STREAMING_URL] [STREAMING_KEY_REGEX] [PUBLIC_URL_REGEX]

Arguments:
  <NAME>
          The name of the streaming service

  <KIND>
          The kind of the streaming service

          Possible values:
          - builtin:  The built-in kind
          - custom:   A custom kind
          - provider: A provider kind

  [STREAMING_URL]
          The endpoint url of the streaming service (for `provider` kind only)

  [STREAMING_KEY_REGEX]
          The format of the streaming key (for `provider` kind only)

  [PUBLIC_URL_REGEX]
          The format of the url from which the stream can be accessed (for `provider` kind only)

Options:
  -h, --help
          Print help (see a summary with '-h')
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-streaming-services-create -->

#### Delete an Existing Streaming Service

Run `opentalk-controller streaming-services delete <StreamingServiceId>` to delete an existing streaming service.

<!-- begin:fromfile:text:cli-usage/opentalk-controller-streaming-services-delete -->

```text
Delete a streaming service by id

Usage: opentalk-controller streaming-services delete <ID>

Arguments:
  <ID>  The Id of the streaming service to delete

Options:
  -h, --help  Print help
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-streaming-services-delete -->

#### Edit an Existing Streaming Service

Run `opentalk-controller streaming-services edit <StreamingServiceId>` to edit an existing streaming service.

Help output looks like this:

<!-- begin:fromfile:text:cli-usage/opentalk-controller-streaming-services-edit -->

```text
Modify an existing streaming service

Usage: opentalk-controller streaming-services edit [OPTIONS] <ID>

Arguments:
  <ID>
          The Id of the streaming service to modify

Options:
      --set-name <SET_NAME>
          Set a new name

      --set-kind <SET_KIND>
          Set the kind

          Possible values:
          - builtin:  The built-in kind
          - custom:   A custom kind
          - provider: A provider kind

      --set-streaming-url <SET_STREAMING_URL>
          Set the endpoint url (for `provider` kind only)

      --set-streaming-key-regex <SET_STREAMING_KEY_REGEX>
          Set the format of the streaming key (for `provider` kind only)

      --set-public-url-regex <SET_PUBLIC_URL_REGEX>
          Set the format of the url from which the stream can be accessed (for `provider` kind only)

  -h, --help
          Print help (see a summary with '-h')
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-streaming-services-edit -->

These subcommand options enable the modification of streaming service names and kinds as well as streaming and public endpoint values.
