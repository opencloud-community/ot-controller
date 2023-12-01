---
sidebar_position: 114
---

# Metrics

The OpenTalk controller is collecting metrics and is exposing them through the `/metrics` endpoint.

## Configuration

The section in the [configuration file](configuration.md) is called `metrics`.

By default, the `/metrics` endpoint refuses all connections. The access can be configured with an allowlist.

| Field       | Type     | Required | Default value | Description                                                       |
| ----------- | -------- | -------- | ------------- | ----------------------------------------------------------------- |
| `allowlist` | `string` | no       | -             | List of IP-Addresses or Subnet which are allowed to fetch metrics |

### Examples

#### Access Denied (default)

```toml
[metrics]
allowlist = []
```

#### Only allow localhost

```toml
[metrics]
allowlist = ["127.0.0.0/8", "::ffff:0:0/96"]
```

#### Allow IPs and Subnets

```toml
[metrics]
allowlist = ["1.1.1.1", "127.0.0.0/8"]
```

## Web-API

The metrics can be accessed via the `/metrics` endpoint in the [OpenMetrics Text Format](https://github.com/OpenObservability/OpenMetrics), which is utilized by [prometheus](https://prometheus.io/docs/instrumenting/exposition_formats/#openmetrics-text-format).

### Metrics Exposed

| Key                                              | Type      | Labels                  | Description                                                     |
| ------------------------------------------------ | --------- | ----------------------- | --------------------------------------------------------------- |
| web_request_durations_bucket                     | histogram | method, handler, status | summary of request durations                                    |
| web_response_sizes_bucket                        | histogram | method, handler, status | summary of response sizes                                       |
| web_issued_email_tasks_count_bucket              | counter   | mail_task_kind          | Number of issued email tasks                                    |
| signaling_runner_startup_time_seconds_bucket     | histogram | successful              | Time the runner takes to initialize                             |
| signaling_runner_destroy_time_seconds_bucket     | histogram | successful              | Time the runner takes to stop                                   |
| signaling_created_rooms_count_bucket             | counter   |                         | Number of created rooms                                         |
| signaling_destroyed_rooms_count_bucket           | counter   |                         | Number of destroyed rooms                                       |
| signaling_participants_count_bucket              | gauge     | participation_kind      | Number of participants                                          |
| signaling_participants_with_audio_count_bucket   | gauge     | media_session_type      | Number of participants with audio unmuted                       |
| signaling_participants_with_video_count_bucket   | gauge     | media_session_type      | Number of participants with video unmuted                       |
| sql_dbpool_connections_bucket                    | gauge     |                         | Number of currently non-idling db connections                   |
| sql_dbpool_connections_idle_bucket               | gauge     |                         | Number of currently idling db connections                       |
| sql_execution_time_seconds_bucket                | histogram |                         | SQL query execution time for whole queries during web operation |
| sql_errors_total_bucket                          | counter   |                         | Counter of SQL errors                                           |
| redis_command_execution_time_seconds_bucket      | histogram | command                 | Redis command execution time                                    |
| kustos_enforce_execution_time_seconds_bucket     | histogram |                         | Kustos enforce execution time                                   |
| kustos_load_policy_execution_time_seconds_bucket | histogram |                         | Kustos load policy execution time                               |
