# Meeting Reports (Terdoc)

The OpenTalk Controller uses [terdoc](https://gitlab.opencode.de/opentalk/terdoc)
to generate meeting reports. The structure and content of meeting reports can be
customized by providing a template in the controller configuration.

## Configuration

The section in the [configuration file](configuration.md) is called `reports`.

| Field      | Type     | Required | Description                                                                                                    |
| ---------- | -------- | -------- | -------------------------------------------------------------------------------------------------------------- |
| `url`      | `string` | yes      | The URL where the terdoc service can be reached                                                                |
| `template` | `string` | yes      | The template which should be used for the report generation (see [Report Template](#meeting-report-template)). |

### Meeting Report Template

The meeting report template uses the [Tera](https://keats.github.io/tera/docs/) templating language.
For more information on how to use this templating language, refer to the [official Tera documentation](https://keats.github.io/tera/docs/#introduction).

The controller provides a collection of variables that can be used inside the template:

| Name           | type     | Description                                                                                             |
| -------------- | -------- | ------------------------------------------------------------------------------------------------------- |
| `title`        | `string` | The title of the meeting                                                                                |
| `description`  | `string` | The description of the meeting                                                                          |
| `starts_at`    | `string` | The start time when the meeting was schedules. This might be empty if the meeting is an ad-hoc meeting. |
| `starts_at_tz` | `string` | The time zone of the start time.                                                                        |
| `ends_at`      | `string` | The end time when the meeting was scheduled. This might be empty if the meeting is an ad-hoc meeting.   |
| `ends_at_tz`   | `string` | The time zone of the start time.                                                                        |
| `participants` | `array`  | An list of all [participants](#participant) of the meeting.                                             |

#### Participant

| Name        | type     | Description                                                                                                                        |
| ----------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `id`        | `uuid`   | The participant ID. The same user can join the same meeting multiple times. This ID is unique for each participant in the meeting. |
| `name`      | `string` | The name of the participant.                                                                                                       |
| `role`      | `string` | The role of the participant. This might me `                                                                                       |
| `kind`      | `string` | either `"user"`, `"guest"` or `"moderator"`                                                                                        |
| `email`     | `string` | E-Mail address of the user                                                                                                         |
| `joined_at` | `string` | timestamp of when the participant joined                                                                                           |
| `left_at`   | `string` | timestamp of when the participant left the room                                                                                    |

### Examples

#### Plain HTTP

```toml
[reports]
url = "https://terdoc.example.com"
template.inline = """
# The Meeting Report Template

...
"""
```
