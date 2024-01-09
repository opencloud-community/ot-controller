---
sidebar_position: 107
title: Execution of maintenance jobs
---

# Jobs

Jobs can be executed through the command-line in order to perform maintenance tasks. Currently, these jobs are not executed automatically and require manual setup by the administrator. They can be executed via cronjobs or systemd timers, for example.

## Available jobs

### Job: `self-check`

This job does not perform any changes to the system, it simply prints a few log
messages of different severity. Use this job to verify that job execution works
as expected without performing any changes.

Execute with:

```text
opentalk-controller jobs execute self-check
```

Example output:

<!-- begin:fromfile:text:cli-usage/opentalk-controller-jobs-execute-self-check -->

```text
[INFO ] Starting job execution
[INFO ] Loading parameters
[INFO ] Executing self-check job
[INFO ]
[TRACE] Test output in TRACE level
[DEBUG] Test output in DEBUG level
[INFO ] Test output in INFO level
[WARN ] Test output in WARN level
[ERROR] Test output in ERROR level
[INFO ]
[INFO ] Job finished successfully
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-jobs-execute-self-check -->

#### Parameters

The job takes an empty JSON object as its parameter.

The default parameters for the job look like this:

<!-- begin:fromfile:json:jobs/parameters-self-check.json -->

```json
{}
```

<!-- end:fromfile:json:jobs/parameters-self-check.json -->

### Job: `event-cleanup`

This job is intended to delete non-recurring events that have ended a certain
duration ago. Its use case is to ensure GDPR compliance.

In addition, all data associated with the event will be deleted, such as:

- assets stored in the [storage system](minio.md)
- folders including the share links from a [shared folder system](shared_folder.md)
- all database content belonging to the event (e.g. invites)
- the room which was associated to the event

#### Parameters

The job takes a JSON object with the following fields as a parameter. All
fields are optional, if any of them is not included in the parameter object, the
default value will be used.

| Field                                  | Type   | Default value | Description                                                                                                                                                    |
| -------------------------------------- | ------ | ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `days_since_last_occurrence`           | `uint` | `30`          | The minimum number of days that must have passed since the end of an event for it to be deleted                                                                |
| `fail_on_shared_folder_deletion_error` | `bool` | `false`       | When `true`, the job will consider failure during deletion of the shared folder an error and abort, otherwise it is considered a warning and the job continues |

The default parameters for the job look like this:

<!-- begin:fromfile:json:jobs/parameters-event-cleanup.json -->

```json
{
  "days_since_last_occurrence": 30,
  "fail_on_shared_folder_deletion_error": false
}
```

<!-- end:fromfile:json:jobs/parameters-event-cleanup.json -->

### Job: `adhoc-event-cleanup`

This job is intended to delete adhoc events that were created a certain
duration ago. Its use case is to ensure GDPR compliance.

In addition, all data associated with the event will be deleted, such as:

- assets stored in the [storage system](minio.md)
- folders including the share links from a [shared folder system](shared_folder.md)
- all database content belonging to the event (e.g. invites)
- the room which was associated to the event

#### Parameters

The job takes a JSON object with the following fields as a parameter. All
fields are optional, if any of them is not included in the parameter object, the
default value will be used.

| Field                                  | Type   | Default value | Description                                                                                                                                                    |
| -------------------------------------- | ------ | ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fail_on_shared_folder_deletion_error` | `bool` | `false`       | When `true`, the job will consider failure during deletion of the shared folder an error and abort, otherwise it is considered a warning and the job continues |
| `seconds_since_creation`               | `uint` | `86400`       | The minimum number of seconds since the creation of the adhoc event                                                                                            |

The default parameters for the job look like this:

<!-- begin:fromfile:json:jobs/parameters-adhoc-event-cleanup.json -->

```json
{
  "fail_on_shared_folder_deletion_error": false,
  "seconds_since_creation": 86400
}
```

<!-- end:fromfile:json:jobs/parameters-adhoc-event-cleanup.json -->

### Job: `invite-cleanup`

This job is intended to clear REST API permissions for inactive or expired invites to events.

#### Parameters

The job takes a JSON object with the following fields as a parameter. All
fields are optional, if any of them is not included in the parameter object, the
default value will be used.

| Field            | Type               | Default value | Description                                                                                                                                                         |
| ---------------- | ------------------ | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `expired_before` | `string` or `null` | `null`        | Timestamp in ISO 8601 format. All invites that expired before this timestamp are deleted by this job. If no timestamp is provided, the current system time is used. |

The default parameters for the job look like this:

<!-- begin:fromfile:json:jobs/parameters-invite-cleanup.json -->

```json
{
  "expired_before": null
}
```

<!-- end:fromfile:json:jobs/parameters-invite-cleanup.json -->

## `opentalk-controller jobs` subcommand

This subcommand is the top-level entrypoint to manage and execute maintenance jobs.

Help output looks like this:

<!-- begin:fromfile:text:cli-usage/opentalk-controller-jobs-help -->

```text
Manage and execute maintenance jobs

Usage: opentalk-controller jobs <COMMAND>

Commands:
  execute             Execute a job by its job type id
  default-parameters  Show the default parameter set for a job
  help                Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-jobs-help -->

## `opentalk-controller jobs execute` subcommand

This subcommand is used to execute maintenance jobs.

If a job fails to execute, a corresponding error message is printed to `stderr`,
and the command exits with a non-zero exit code.

Help output looks like this:

<!-- begin:fromfile:text:cli-usage/opentalk-controller-jobs-execute-help -->

```text
Execute a job by its job type id

Usage: opentalk-controller jobs execute [OPTIONS] <JOB_TYPE>

Arguments:
  <JOB_TYPE>
          The type of the job to be executed

          Possible values:
          - self-check:          A simple self-check of the job execution system
          - event-cleanup:       A job for cleaning up events that ended at minimum a defined duration ago
          - adhoc-event-cleanup: A job to cleanup adhoc events a certain duration after they were created
          - invite-cleanup:      A job for cleaning up expired invites

Options:
      --parameters <PARAMETERS>
          The parameters that the job uses when executed, encoded in a valid JSON object.

          When not provided, this will be an empty JSON object. That means each job will fill in its own default parameter object fields. The default parameters for a job can be queried using the `jobs default-parameters <JOB_TYPE>` subcommand.

          [default: {}]

      --timeout <TIMEOUT>
          Timeout after which the job execution gets aborted, in seconds

          [default: 3600]

      --hide-duration
          Don't show the duration it took to run a job. Useful for generating reproducible output

  -h, --help
          Print help (see a summary with '-h')
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-jobs-execute-help -->

## `opentalk-controller jobs default-parameters` subcommand

This subcommand is used to query the default parameters used when executing maintenance jobs.

Help output looks like this:

<!-- begin:fromfile:text:cli-usage/opentalk-controller-jobs-default-parameters-help -->

```text
Show the default parameter set for a job

Usage: opentalk-controller jobs default-parameters <JOB_TYPE>

Arguments:
  <JOB_TYPE>
          The type of the job for which the parameters should be shown

          The parameters are shown in plain pretty-printed JSON

          Possible values:
          - self-check:          A simple self-check of the job execution system
          - event-cleanup:       A job for cleaning up events that ended at minimum a defined duration ago
          - adhoc-event-cleanup: A job to cleanup adhoc events a certain duration after they were created
          - invite-cleanup:      A job for cleaning up expired invites

Options:
  -h, --help
          Print help (see a summary with '-h')
```

<!-- end:fromfile:text:cli-usage/opentalk-controller-jobs-default-parameters-help -->
