# Control

The control module contains the base functionality for the conference including information like the participant list,
roles and hand raise.

## Commands

### Join

Must be the first message to be sent when the websocket connection is established.

#### Fields

| Field          | Type     | Required | Description                                                   |
| -------------- | -------- | -------- | ------------------------------------------------------------- |
| `action`       | `enum`   | yes      | Must be `"join"`                                              |
| `display_name` | `string` | yes      | String to be displayed as the user's display name in the room |

##### Example

```json
{
    "action": "join",
    "display_name": "Test"
}
```

---

### EnterRoom

Can only be sent while in the waiting room and after being accepted by a moderator.

See [InWaitingRoom](moderation#inwaitingroom)

#### Fields

| Field    | Type   | Required | Description            |
| -------- | ------ | -------- | ---------------------- |
| `action` | `enum` | yes      | Must be `"enter_room"` |

##### Example

```json
{
    "action": "enter_room"
}
```

---

### Raise Hand

Notify other participants that a participant's hand is raised.

#### Fields

| Field    | Type   | Required | Description            |
| -------- | ------ | -------- | ---------------------- |
| `action` | `enum` | yes      | Must be `"raise_hand"` |

##### Example

```json
{
    "action": "raise_hand"
}
```

---

### Lower Hand

Notify other participants that a participant's hand is no longer raised.

#### Fields

| Field    | Type   | Required | Description            |
| -------- | ------ | -------- | ---------------------- |
| `action` | `enum` | yes      | Must be `"raise_hand"` |

##### Example

```json
{
    "action": "raise_hand"
}
```

---

### Grant moderator role

Requires moderator role.

Grant a participant the moderator role.

#### Fields

| Field    | Type     | Required | Description                                       |
| -------- | -------- | -------- | ------------------------------------------------- |
| `action` | `enum`   | yes      | Must be `"grant_moderator_role"`                  |
| `target` | `string` | yes      | Id of the participant to grant the moderator role |

##### Example

```json
{
    "action": "grant_moderator_role",
    "target": "00000000-0000-0000-0000-000000000000"
}
```

---

### Revoke moderator role

Requires moderator role.

Revoke the moderator role from a participant. Cannot revoke the moderator role from the creator of the room.

#### Fields

| Field    | Type     | Required | Description                                             |
| -------- | -------- | -------- | ------------------------------------------------------- |
| `action` | `enum`   | yes      | Must be `"grant_moderator_role"`                        |
| `target` | `string` | yes      | Id of the participant to revoke the moderator role from |

##### Example

```json
{
    "action": "revoke_moderator_role",
    "target": "00000000-0000-0000-0000-000000000000"
}
```

---

## Events

### Data Types

#### Participant

##### Fields

Information about another participant

| Field                  | Type          | Always | Description                                                                                 |
| ---------------------- | ------------- | ------ | ------------------------------------------------------------------------------------------- |
| `id`                   | `string`      | yes    | ID of the participant                                                                       |
| `control`              | `ControlData` | yes    | Information about the participant provided by the control module                            |
| `some_other_module...` | `object`      | ?      | Not an actual field but every module can attach data about the participant under their name |

#### ControlData

Information about another participant provided by the `control` module

##### Fields

| Field                | Type     | Always | Description                                                     |
| -------------------- | -------- | ------ | --------------------------------------------------------------- |
| `display_name`       | `string` | yes    | Display name of the participant                                 |
| `role`               | `enum`   | yes    | either `"guest,`, `"user"` or `"moderator"`                     |
| `avatar_url`         | `string` | no     | url to your avatar image if the participant is a logged in user |
| `participation_kind` | `enum`   | yes    | either `"user"`, `"guest"` or `"sip"`                           |
| `hand_is_up`         | `bool`   | yes    | true if the user is currently raising his hand                  |
| `joined_at`          | `string` | yes    | timestamp of when the participant joined                        |
| `left_at`            | `string` | no     | timestamp of when the participant left the room                 |
| `hand_updated_at`    | `string` | yes    | timestamp of when the hand-raise status last changed            |
| `is_room_owner`      | `bool`   | yes    | true if the user is the owner of the room                       |

#### EventInfo

##### Fields

Information about the event associated with a room.

| Field      | Type     | Always | Description                          |
| ---------- | -------- | ------ | ------------------------------------ |
| `id`       | `string` | yes    | Id of the event                      |
| `title`    | `string` | yes    | Title of the event                   |
| `is_adhoc` | `bool`   | yes    | True if the event was created ad-hoc |

### JoinSuccess

Received after joining the room. Can be triggered by either calling [Join](#join) or [EnterRoom](#enterroom).

#### Fields

| Field          | Type            | Always | Description                                                                                |
| -------------- | --------------- | ------ |--------------------------------------------------------------------------------------------|
| `message`      | `enum`          | yes    | Is `"join_success"`                                                                        |
| `id`           | `string`        | yes    | Your participant id in this session                                                        |
| `display_name` | `string`        | yes    | Your display name in this session                                                          |
| `avatar_url`   | `string`        | no     | Url to your avatar image if logged                                                         |
| `role`         | `enum`          | yes    | Either `"guest"`, `"user"` or `"moderator"`                                                |
| `closes_at`    | `string`        | no     | The point in time the room closes                                                          |
| `tariff`       | `Tariff`        | yes    | Tariff information, including `quotas` and `modules`                                       |
| `participants` | `Participant[]` | yes    | List of participants in the room                                                           |
| `event_info`   | `EventInfo`     | no     | Information about the event associated with the meeting room. See: [EventInfo](#eventinfo) |

##### Example

```json
{
  "message": "join_success",
  "id": "00000000-0000-0000-0000-000000000000",
  "display_name": "My Display Name",
  "avatar_url":"https://example.org/",
  "role": "moderator",
  "closes_at": "2023-03-10T16:52:54Z",
  "is_room_owner": true,
  "tariff": {
    "id": "53db7cb1-12af-4715-9b17-4a5a57876085",
    "name": "OpenTalkDefaultTariff",
    "quotas": {},
    "enabled_modules": ["core","chat"],
    "disabled_features": ["core::feature1","feature1"],
    "modules":{
      "core": {"features": ["feature2", "feature3"]},
      "chat": {}
    }
  },
  "participants": [
    {
      "id": "00000000-0000-0000-0000-000000000000",
      "control": {
        "display_name": "Someone Else",
        "hand_is_up": false,
        "hand_updated_at": "2022-05-10T10:40:39Z",
        "joined_at": "2022-05-10T10:40:39Z",
        "left_at": "2022-05-10T10:40:42Z",
        "participation_kind": "user",
        "is_room_owner": false
      }
    }
  ],
  "event_info": {
    "id": "fa31b241-612d-4524-930e-b5b0af12acb1"
    "title": "Daily",
  }
}
```

### JoinBlocked

If a tariff is configured for a room, an issued [Join](#join) action may result in this event.

#### Fields

| Field     | Type   | Always | Description                      |
| --------- | ------ | ------ | -------------------------------- |
| `message` | `enum` | yes    | Is `"join_blocked"`               |
| `reason`  | `enum` | yes    | Is `"participant_limit_reached"` |

##### Example

```json
{
    "message": "join_blocked",
    "reason": "participant_limit_reached"
}
```

### Update

Received when a participant changes his state. Wraps a [Participant](#participant).

#### Fields

| Field     | Type   | Always | Description   |
| --------- | ------ | ------ | ------------- |
| `message` | `enum` | yes    | Is `"update"` |

##### Example

```json
{
    "message": "update",
    "id": "00000000-0000-0000-0000-000000000000",
    "control": {
      "display_name": "Someone Else",
      "hand_is_up": false,
      "hand_updated_at": "2022-05-10T10:40:39Z",
      "joined_at": "2022-05-10T10:40:39Z",
      "left_at": "2022-05-10T10:40:42Z",
      "participation_kind": "user"
    }
}
```

### Joined

Received when a participant joined the room. Wraps a [Participant](#participant).

#### Fields

| Field     | Type   | Always | Description   |
| --------- | ------ | ------ | ------------- |
| `message` | `enum` | yes    | Is `"joined"` |

##### Example

```json
{
    "message": "joined",
    "id": "00000000-0000-0000-0000-000000000000",
    "control": {
      "display_name": "Someone Else",
      "hand_is_up": false,
      "hand_updated_at": "2022-05-10T10:40:39Z",
      "joined_at": "2022-05-10T10:40:39Z",
      "left_at": "2022-05-10T10:40:42Z",
      "participation_kind": "user"
    }
}
```

### Left

Received when a participant left the room.

#### Fields

| Field     | Type     | Always | Description                         |
| --------- | -------- | ------ | ----------------------------------- |
| `message` | `enum`   | yes    | Is `"left"`                         |
| `id`      | `string` | yes    | Id of the participant that has left |

##### Example

```json
{
    "message": "left",
    "id": "00000000-0000-0000-0000-000000000000"
}
```

### TimeLimitQuotaElapsed

Received when the quota's time limit has elapsed.

#### Fields

| Field     | Type   | Always | Description                               |
| ----------| ------ | ------ | ----------------------------------------- |
| `message` | `enum` | yes    | Is `"time_limit_quota_elapsed"`           |

### RoleUpdated

Received when a moderator assigned a new role to a participant.

#### Fields

| Field      | Type   | Always | Description                               |
| ---------- | ------ | ------ | ----------------------------------------- |
| `message`  | `enum` | yes    | Is `"role_updated"`                       |
| `new_role` | `enum` | yes    | either `"guest"`, `"user"`, `"moderator"` |

##### Example

```json
{
    "message": "role_updated",
    "new_role": "moderator"
}
```

### RoomDeleted

Received by a participant if removed from the room because the room has been deleted. Will be the last message before server-side websocket disconnection.

#### Fields

| Field     | Type   | Always | Description         |
| ----------| ------ | ------ | ------------------- |
| `message` | `enum` | yes    | Is `"room_deleted"` |

### Error

Received when something went wrong.

#### Fields

| Field     | Type   | Always | Description          |
| --------- | ------ | ------ | -------------------- |
| `message` | `enum` | yes    | Is `"error"`         |
| `text`    | `enum` | yes    | arbitrary error text |

##### Example

```json
{
    "message": "error",
    "text": "something happened"
}
```
