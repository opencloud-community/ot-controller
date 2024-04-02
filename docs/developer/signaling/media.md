# Media

## Overview

The media module is responsible for establishing WebRTC sessions between the client and SFU used by OpenTalk. SFU stands
for Selective Forwarding Unit and is the redistributor of the media published by each participant.

"Publishing media" means to transmit video and audio to OpenTalk via a WebRTC `sendonly` session. Depending on the
use-case the media module allows for 2 different kinds of publishing sessions:

- `video`: (Lower bitrate) Usually low resolution and higher framerate video used for Webcams or similar with more motion
- `screen`: (Higher bitrate) Usually high resolution and low framerate ideal for screen share/presentations with text which must be readable

"Subscribing" to a peer in a conference means to receive video and audio via a WebRTC `recvonly` session. The published
media of another participant is stored within the module-specific data in the [`Participant`](control.md#participant).

The notion of `presenter` is used to communicate screen share permissions.

## Joining the room

### JoinSuccess

When joining a room, the `join_success` control event contains the module-specific fields described below.

#### Fields

| Field          | Type                                                      | Always | Description                                                            |
| -------------- | --------------------------------------------------------- | ------ | ---------------------------------------------------------------------- |
| `is_presenter` | `bool`                                                    | yes    | Represents if the current participant has permissions for screen share |
| `speakers`     | (`ParticipantSpeakingState[]`)(#ParticipantSpeakingState) | yes    | The list of current or previous speakers in the meeting                |
| `force_mute`   | [`ForceMuteState`](#ForceMuteState)                       | yes    | The current force mute state                                           |

##### Example

```json
{
    "is_presenter": true,
    "speakers": [
        {
            "participant": "6802d547-06ff-493a-bcce-da7f3bc37248",
            "is_speaking": false,
            "updated_at": "2023-01-13T12:29:01Z"
        },
        {
            "participant": "0605e657-27b8-443e-81de-31d5921e9a42",
            "is_speaking": true,
            "updated_at": "2023-01-13T12:37:42Z"
        },
        {
            "participant": "69068e53-eb60-4c1d-bfc1-fc31a0dc45a3",
            "is_speaking": false,
            "updated_at": "2023-01-13T12:22:08Z"
        }
    ],
    "force_mute": {
        "type": "enabled",
        "allow_list": [
            "07d32d3e-9510-49bf-82b7-e21fef9db120"
        ]
    }
}
```

### Joined

When joining a room, the `joined` control event sent to all other participants contains the module-specific fields described below.

#### Fields

| Field          | Type                    | Always | Description                                                                          |
| -------------- | ----------------------- | ------ | ------------------------------------------------------------------------------------ |
| `state`        | `ParticipantMediaState` | no     | An object describing the current availability of media and their current mute status |
| `is_presenter` | `bool`                  | yes    | Represents if the other participant has permissions for screen share                 |

__ParticipantMediaState:__

This object represents the __available__ media sessions and encapsulates their mute state.

| Field    | Type                                      | Always | Description                                                                                                        |
| -------- | ----------------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------------ |
| `video`  | [`MediaSessionState`](#mediasessionstate) | no     | If this field is set, the participant is publishing a video, usually a webcam (lower resolution, higher framerate) |
| `screen` | [`MediaSessionState`](#mediasessionstate) | no     | If this field is set, the participant is publishing their screen (usually high resolution, low framerate)          |

##### Example

This example shows a participant's state when publishing only their screen without audio enabled.

```json
{
    "state": {
        "screen": {
            "video": true,
            "audio": false
        }
    },
    "is_presenter": true
}
```

## Commands

<!-- COMMAND PUBLISH -->

### Publish

Create a WebRTC publish session by sending an SDP offer.

#### Response

A [SdpAnswer](#sdpanswer) will return an SDP response.

#### Fields

| Field                | Type     | Required | Description             |
| -------------------- | -------- | -------- | ----------------------- |
| `action`             | `enum`   | yes      | Must be `"publish"`     |
| `sdp`                | `string` | yes      | SDP Offer as a string   |
| `target`             | `string` | yes      | See [`Target`](#target) |
| `media_session_type` | `enum`   | yes      | See [`Target`](#target) |

##### Example

```json
{
    "action": "publish",
    "sdp": "v=0,....",
    "target": "07d32d3e-9510-49bf-82b7-e21fef9db120",
    "media_session_type": "video"
}
```

<!-- COMMAND PUBLISH COMPLETE -->

### PublishComplete

Signal that the WebRTC publish process is complete and set the initial mute status. Other participants will be notified
by an [`Update`](control.md#update) that the media is now available to be subscribed to and if the audio or video track
is muted.
If the force-mute state is enabled, the `media_session_state.video.audio` field must be set to `false` unless the
participant is included in the `allow_list`.

#### Fields

| Field                 | Type                                      | Required | Description                    |
| --------------------- | ----------------------------------------- | -------- | ------------------------------ |
| `action`              | `enum`                                    | yes      | Must be `"publish_complete"`   |
| `media_session_type`  | `enum`                                    | yes      | Either `"video"` or `"screen"` |
| `media_session_state` | [`MediaSessionState`](#mediasessionstate) | yes      |                                |

##### Example

```json
{
    "action": "publish_complete",
    "media_session_type": "video",
    "media_session_state": {
        "video": {
            "video": true,
            "audio": false
        }
    }
}
```

<!-- COMMAND UNPUBLISH -->

### Unpublish

Remove/stop an existing WebRTC publish session. The WebRTC session will instantly be stopped (if not already) by the
SFU when sending this command. Other participants will be notified by an [`Update`](control.md#update) that the published
media is no longer available.

#### Fields

| Field                | Type   | Required | Description                    |
| -------------------- | ------ | -------- | ------------------------------ |
| `action`             | `enum` | yes      | Must be `"unpublish"`          |
| `media_session_type` | `enum` | yes      | Either `"video"` or `"screen"` |

##### Example

```json
{
    "action": "unpublish",
    "media_session_type": "video"
}
```

<!-- COMMAND SUBSCRIBE -->

### Subscribe

Request an SDP offer for the specified target (a peer participant's published media).

#### Response

A [SdpOffer](#SdpOffer) will return an SDP offer to which the client must respond to with an [SdpAnswer](#SdpAnswer).

#### Fields

| Field                | Type     | Required | Description                                                              |
| -------------------- | -------- | -------- | ------------------------------------------------------------------------ |
| `action`             | `enum`   | yes      | Must be `"subscribe"`                                                    |
| `target`             | `string` | yes      | See [`Target`](#target)                                                  |
| `media_session_type` | `enum`   | yes      | See [`Target`](#target)                                                  |
| `without_video`      | `bool`   | no       | Can be used to opt out of the video stream for the entire WebRTC session |

##### Example

```json
{
    "action": "subscribe",
    "target": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video",
    "without_video": true
}
```

<!-- COMMAND RESUBSCRIBE -->

### Resubscribe

Request for a WebRTC subscribe session to be restarted, this will restart the complete negotiation process.

#### Response

A [SdpOffer](#SdpOffer) will return an SDP offer to which the client must respond to with an [SdpAnswer](#SdpAnswer).

#### Fields

| Field                | Type     | Required | Description             |
| -------------------- | -------- | -------- | ----------------------- |
| `action`             | `enum`   | yes      | Must be `"resubscribe"` |
| `target`             | `string` | yes      | See [`Target`](#target) |
| `media_session_type` | `enum`   | yes      | See [`Target`](#target) |

##### Example

```json
{
    "action": "resubscribe",
    "target": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video"
}
```

<!-- COMMAND SDP ANSWER -->

### SdpAnswer

Complete the initial SDP negotiation of a subscription by sending an SDP answer. The SFU will begin the WebRTC/ICE
handshake and notify later via an [`WebrtcUp`](#webrtcup) if the establishment was successful from it's side.

#### Fields

| Field                | Type     | Required | Description             |
| -------------------- | -------- | -------- | ----------------------- |
| `action`             | `enum`   | yes      | Must be `"sdp_answer"`  |
| `sdp`                | `string` | yes      | SDP Answer as a string  |
| `target`             | `string` | yes      | See [`Target`](#target) |
| `media_session_type` | `enum`   | yes      | See [`Target`](#target) |

##### Example

```json
{
    "action": "sdp_answer",
    "sdp": "v=0,...",
    "target": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video"
}
```

<!-- COMMAND SDP CANDIDATE -->

### SdpCandidate

Exchange a ICE trickle candidate with a establishing WebRTC session. This is part of the WebRTC establishment and is
forwarded to the SFU.

#### Fields

| Field                | Type                                    | Required | Description               |
| -------------------- | --------------------------------------- | -------- | ------------------------- |
| `action`             | `enum`                                  | yes      | Must be `"sdp_candidate"` |
| `candidate`          | [`TrickleCandidate`](#tricklecandidate) | yes      | Candidate to send         |
| `target`             | `string`                                | yes      | See [`Target`](#target)   |
| `media_session_type` | `enum`                                  | yes      | See [`Target`](#target)   |

##### Example

```json
{
    "action": "sdp_candidate",
    "candidate": {
        "sdpMLineIndex": 0,
        "candidate": "candidate:..."
    },
    "target": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video"
}
```

<!-- COMMAND SDP END OF CANDIDATES -->

### SdpEndOfCandidates

Signal the establishing WebRTC session that there are no more ICE trickle candidates. This is part of the WebRTC
establishment and is forwarded to the SFU.

#### Fields

| Field                | Type     | Required | Description                       |
| -------------------- | -------- | -------- | --------------------------------- |
| `action`             | `enum`   | yes      | Must be `"sdp_end_of_candidates"` |
| `target`             | `string` | yes      | See [`Target`](#target)           |
| `media_session_type` | `enum`   | yes      | See [`Target`](#target)           |

##### Example

```json
{
    "action": "sdp_end_of_candidates",
    "target": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video"
}
```

<!-- COMMAND UPDATE MEDIA SESSION -->

### UpdateMediaSession

Update the mute state for a WebRTC publish session. This information is forwarded to all other participants using inside
an [`Update`](control.md#update).
If the force-mute state is enabled, the field `media_session_state.audio` must be set to `false` unless the participant
is included in the `allow_list`.

#### Fields

| Field                 | Type                                      | Required | Description                      |
| --------------------- | ----------------------------------------- | -------- | -------------------------------- |
| `action`              | `enum`                                    | yes      | Must be `"update_media_session"` |
| `media_session_type`  | `enum`                                    | yes      | Either `"video"` or `"screen"`   |
| `media_session_state` | [`MediaSessionState`](#mediasessionstate) | yes      | The new state for the session    |

##### Example

```json
{
    "action": "update_media_session",
    "media_session_type": "video",
    "media_session_state": {
        "video": {
            "audio": true,
            "video": true,
        },
        "screen": null
    }
}
```

<!-- COMMAND CONFIGURE -->

### Configure

Configure a WebRTC subscribe session. The request is forwarded to the SFU which configured the media streams
appropriately.

This is used to adjust the quality of a video stream to save on bandwidth.

#### Fields

| Field                | Type                      | Required | Description             |
| -------------------- | ------------------------- | -------- | ----------------------- |
| `action`             | `enum`                    | yes      | Must be `"configure"`   |
| `configuration`      | `SubscriberConfiguration` | yes      |                         |
| `target`             | `string`                  | yes      | See [`Target`](#target) |
| `media_session_type` | `enum`                    | yes      | See [`Target`](#target) |

__SubscriberConfiguration:__

| Field       | Type   | Required | Description                                                                                                                                                                                                                                                                    |
| ----------- | ------ | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `video`     | `bool` | no       | If video transmission should be enabled. This option should be used to turn off videos of participants which are out of view                                                                                                                                                   |
| `substream` | `int`  | no       | Select a different substream if any are available. Substreams are usually the same video with difference in bitrate/quality, where 0 is _usually_ the lowest quality. This options should be used to lower the quality of participants that are displayed in a smaller window. |

##### Example

```json
{
    "action": "configure",
    "configuration": {
        "video": true,
        "substream": 0
    },
    "target": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video",
}
```

<!-- COMMAND GRANT PRESENTER ROLE -->

### GrantPresenterRole

Grant one or more participants the right to show their screen

#### Fields

| Field             | Type     | Required | Description                                         |
| ----------------- | -------- | -------- | --------------------------------------------------- |
| `action`          | `enum`   | yes      | Must be `"grant_presenter_role"`                    |
| `participant_ids` | `string` | yes      | List of participant ids to grant the presenter role |

##### Example

```json
{
    "action": "grant_presenter_role",
    "participant_ids": [
        "84a2c872-94fb-4b41-aca7-13d784c92a72",
        "2375602f-c74c-4935-9933-bfd67d4e8ae5"
    ]
}
```

<!-- COMMAND REVOKE PRESENTER ROLE -->

### RevokePresenterRole

Revoke one or more participants the right to show their screen

#### Fields

| Field             | Type     | Required | Description                                            |
| ----------------- | -------- | -------- | ------------------------------------------------------ |
| `action`          | `enum`   | yes      | Must be `"revoke_presenter_role"`                      |
| `participant_ids` | `string` | yes      | List of participant ids to revoke their presenter role |

##### Example

```json
{
    "action": "revoke_presenter_role",
    "participant_ids": [
        "84a2c872-94fb-4b41-aca7-13d784c92a72",
        "2375602f-c74c-4935-9933-bfd67d4e8ae5"
    ]
}
```

<!-- COMMAND MODERATOR MUTE -->

### ModeratorMute

Request another participant to mute their microphone.

#### Fields

| Field     | Type     | Required | Description                                    |
| --------- | -------- | -------- | ---------------------------------------------- |
| `action`  | `enum`   | yes      | Must be `"moderator_mute"`                     |
| `targets` | `string` | yes      | List of participant ids to request to be muted |
| `force`   | `bool`   | yes      | We're not asking you to mute, we're telling!   |

##### Example

```json
{
    "action": "moderator_mute",
    "targets": [
        "84a2c872-94fb-4b41-aca7-13d784c92a72",
        "2375602f-c74c-4935-9933-bfd67d4e8ae5"
    ],
    "force": true
}
```

<!-- COMMAND MODERATOR ENABLE FORCE MUTE -->

### ModeratorEnableForceMute

Enables the force mute state which mutes all participants and disallows enabling the microphone except for participants included in the `allow_list`.

#### Fields

| Field        | Type       | Required | Description                                               |
| ------------ | ---------- | -------- | --------------------------------------------------------- |
| `action`     | `enum`     | yes      | Must be `moderator_enable_force_mute`                     |
| `allow_list` | `string[]` | yes      | The list of participants that are still allowed to unmute |

#### Example

```json
{
    "action": "moderator_enable_force_mute",
    "allow_list": [
        "84a2c872-94fb-4b41-aca7-13d784c92a72",
        "2375602f-c74c-4935-9933-bfd67d4e8ae5"
    ]
}
```

<!-- COMMAND MODERATOR DISABLE FORCE MUTE -->

### ModeratorDisableForceMute

Disables the force-mute state and allows participants to enable their microphones again.

#### Fields

| Field    | Type   | Required | Description                            |
| -------- | ------ | -------- | -------------------------------------- |
| `action` | `enum` | yes      | Must be `moderator_disable_force_mute` |

#### Example

```json
{
    "action": "moderator_disable_force_mute"
}
```

<!-- COMMAND UPDATE SPEAKING STATE -->

### UpdateSpeakingState

Update the client's speaking state. This should be sent to the service by the
client when it detects either start or end of a speaking period.

After receiving this message, the service sends out a
[`SpeakerUpdated`](#speakerupdated) event to all participants, including the
one that changed their state.

#### Fields

| Field         | Type   | Required | Description                                                                              |
| ------------- | ------ | -------- | ---------------------------------------------------------------------------------------- |
| `action`      | `enum` | yes      | Must be `"update_speaking_state"`                                                        |
| `is_speaking` | `bool` | yes      | The flag indicating whether the client states that the participant is currently speaking |

##### Example

```json
{
    "action": "update_speaking_state",
    "is_speaking": true
}
```

## Events

<!-- EVENT SDP ANSWER -->

### SdpAnswer

Provides an SDP answer string in response to a [`Publish`](#publish) command.

#### Fields

| Field                | Type     | Always | Description             |
| -------------------- | -------- | ------ | ----------------------- |
| `message`            | `enum`   | yes    | Is `"sdp_answer"`       |
| `sdp`                | `string` | yes    | SDP Answer in a string  |
| `source`             | `string` | yes    | See [`Source`](#source) |
| `media_session_type` | `enum`   | yes    | See [`Source`](#source) |

##### Example

```json
{
    "message": "sdp_answer",
    "sdp": "v=0,...",
    "source": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video"
}
```

<!-- EVENT SDP OFFER -->

### SdpOffer

Returns an SDP offer string in response to a [`Subscribe`](#subscribe) command.

#### Fields

| Field                | Type     | Always | Description             |
| -------------------- | -------- | ------ | ----------------------- |
| `message`            | `enum`   | yes    | Is `"sdp_offer"`        |
| `sdp`                | `string` | yes    | SDP Answer in a string  |
| `source`             | `string` | yes    | See [`Source`](#source) |
| `media_session_type` | `enum`   | yes    | See [`Source`](#source) |

##### Example

```json
{
    "message": "sdp_offer",
    "sdp": "v=0,...",
    "source": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video"
}
```

<!-- EVENT SDP CANDIDATE -->

### SdpCandidate

Receive an SDP candidate for the specified WebRTC session.

#### Fields

| Field                | Type                                    | Always | Description             |
| -------------------- | --------------------------------------- | ------ | ----------------------- |
| `message`            | `enum`                                  | yes    | Is `"sdp_candidate"`    |
| `candidate`          | [`TrickleCandidate`](#tricklecandidate) | yes    |                         |
| `source`             | `string`                                | yes    | See [`Source`](#source) |
| `media_session_type` | `enum`                                  | yes    | See [`Source`](#source) |

##### Example

```json
{
    "message": "sdp_candidate",
    "candidate": {
        "sdpMLineIndex": 0,
        "candidate": "candidate:..."
    },
    "source": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video"
}
```

<!-- EVENT SDP END OF CANDIDATES -->

### SdpEndOfCandidates

Receive an SDP end-of-candidates signal for the specified WebRTC session.

#### Fields

| Field                | Type     | Always | Description                  |
| -------------------- | -------- | ------ | ---------------------------- |
| `message`            | `enum`   | yes    | Is `"sdp_end_of_candidates"` |
| `source`             | `string` | yes    | See [`Source`](#source)      |
| `media_session_type` | `enum`   | yes    | See [`Source`](#source)      |

##### Example

```json
{
    "message": "sdp_end_of_candidates",
    "source": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video"
}
```

<!-- EVENT WEBRTC UP -->

### WebrtcUp

The SFU reports that the specified WebRTC session has been established.

#### Fields

| Field                | Type     | Always | Description             |
| -------------------- | -------- | ------ | ----------------------- |
| `message`            | `enum`   | yes    | Is `"webrtc_up"`        |
| `source`             | `string` | yes    | See [`Source`](#source) |
| `media_session_type` | `enum`   | yes    | See [`Source`](#source) |

##### Example

```json
{
    "message": "webrtc_up",
    "source": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video"
}
```

<!-- EVENT WEBRTC DOWN -->

### WebrtcDown

The SFU reports that the specified WebRTC session has broken down.

#### Fields

| Field                | Type     | Always | Description             |
| -------------------- | -------- | ------ | ----------------------- |
| `message`            | `enum`   | yes    | Is `"webrtc_down"`      |
| `source`             | `string` | yes    | See [`Source`](#source) |
| `media_session_type` | `enum`   | yes    | See [`Source`](#source) |

##### Example

```json
{
    "message": "webrtc_down",
    "source": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video"
}
```

<!-- EVENT WEBRTC SLOW -->

### WebrtcSlow

The SFU reports problems sending/receiving data for the specified WebRTC session

#### Fields

| Field                | Type     | Always | Description                       |
| -------------------- | -------- | ------ | --------------------------------- |
| `message`            | `enum`   | yes    | Is `"webrtc_slow"`                |
| `direction`          | `enum`   | yes    | Either `upstream` or `downstream` |
| `source`             | `string` | yes    | See [`Source`](#source)           |
| `media_session_type` | `enum`   | yes    | See [`Source`](#source)           |

##### Example

```json
{
    "message": "webrtc_slow",
    "direction": "upstream",
    "source": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video"
}
```

<!-- EVENT MEDIA STATUS -->

### MediaStatus

The SFU reports an update in the current status of the specified WebRTC session

#### Fields

| Field                | Type     | Always | Description                                                                                                                       |
| -------------------- | -------- | ------ | --------------------------------------------------------------------------------------------------------------------------------- |
| `message`            | `enum`   | yes    | Is `"media_status"`                                                                                                               |
| `kind`               | `string` | yes    | The kind of media, so usually either `video`, `audio` but might also be application data if the WebRTC sessions are used for that |
| `receiving`          | `bool`   | yes    | Reports the status of the media being received, if false the SFU reports that it no longer receives any data.                     |
| `source`             | `string` | yes    | See [`Source`](#source)                                                                                                           |
| `media_session_type` | `enum`   | yes    | See [`Source`](#source)                                                                                                           |

##### Example

```json
{
    "message": "media_status",
    "kind": "audio",
    "receiving": true,
    "source": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "media_session_type": "video"
}
```

<!-- EVENT PRESENTER GRANTED -->

### PresenterGranted

Presenter rights have been granted

#### Fields

| Field     | Type   | Always | Description              |
| --------- | ------ | ------ | ------------------------ |
| `message` | `enum` | yes    | Is `"presenter_granted"` |

##### Example

```json
{
    "message": "presenter_granted",
}
```

<!-- EVENT PRESENTER REVOKED -->

### PresenterRevoked

Presenter rights have been revoked

#### Fields

| Field     | Type   | Always | Description              |
| --------- | ------ | ------ | ------------------------ |
| `message` | `enum` | yes    | Is `"presenter_revoked"` |

##### Example

```json
{
    "message": "presenter_revoked",
}
```

<!-- EVENT REQUEST MUTE -->

### RequestMute

You are being asked to mute yourself.

#### Fields

| Field     | Type     | Always | Description                                                  |
| --------- | -------- | ------ | ------------------------------------------------------------ |
| `message` | `enum`   | yes    | Is `"request_mute"`                                          |
| `issuer`  | `string` | yes    | Id of the participant which asked you to mute yourself       |
| `force`   | `bool`   | yes    | Clients should automatically mute when this is set to `true` |

##### Example

```json
{
    "message": "request_mute",
    "issuer": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "force": true
}
```

<!-- EVENT FORCE MUTE ENABLED -->

### ForceMuteEnabled

The force-mute state was enabled. Only the participants listed in `allow_list` are allowed to enable their microphones.

| Field        | Type       | Always | Description                                           |
| ------------ | ---------- | ------ | ----------------------------------------------------- |
| `message`    | `enum`     | yes    | Is `"force_mute_enabled"`                             |
| `allow_list` | `string[]` | yes    | List of participants that are still allowed to unmute |

<!-- EVENT FORCE MUTE DISABLED -->

### ForceMuteDisabled

The force-mute state was disabled. All participants are allowed to enabled their microphones.

| Field     | Type   | Always | Description                |
| --------- | ------ | ------ | -------------------------- |
| `message` | `enum` | yes    | Is `"force_mute_disabled"` |

<!-- EVENT SPEAKERS UPDATED -->

### SpeakerUpdated

The speaking state of a participant has been updated because they started or stopped speaking.

#### Fields

| Field         | Type     | Always | Description                                                                              |
| ------------- | -------- | ------ | ---------------------------------------------------------------------------------------- |
| `message`     | `enum`   | yes    | Is `"speaker_updated"`                                                                   |
| `participant` | `string` | yes    | Id of the participant whose speaking state changed                                       |
| `is_speaking` | `bool`   | yes    | The flag indicating whether the client states that the participant is currently speaking |
| `updated_at`  | `string` | yes    | Timestamp of the last change of `is_speaking` for this participant                       |

##### Example

```json
{
    "message": "speaker_updated",
    "participant": "84a2c872-94fb-4b41-aca7-13d784c92a72",
    "is_speaking": true,
    "updated_at": "2023-01-13T12:37:08Z"
}
```

<!-- EVENT ERROR -->

### Error

Something went wrong

#### Fields

| Field                | Type     | Always                                                                   | Description                      |
| -------------------- | -------- | ------------------------------------------------------------------------ | -------------------------------- |
| `message`            | `enum`   | yes                                                                      | Is `"error"`                     |
| `error`              | `enum`   | yes                                                                      | Any of the codes specified below |
| `source`             | `string` | if `error` is `"invalid_request_offer"` or `"invalid_configure_request"` | See [`Source`](#source)          |
| `media_session_type` | `enum`   | if `error` is `"invalid_request_offer"` or `"invalid_configure_request"` | See [`Source`](#source)          |

__Error codes:__

- `"invalid_sdp_offer"`
- `"handle_sdp_answer"`
- `"invalid_candidate"`
- `"invalid_end_of_candidates"`
- `"invalid_request_offer"`
- `"invalid_configure_request"`

- `"permission_denied"`: The requester didn't meet the required permissions for the request

##### Example

```json
{
    "message": "error",
    "error": "invalid_sdp_offer"
}
```

## Common Types

### Source

| Field                | Type     | Always | Description                                                                                                                                 |
| -------------------- | -------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `source`             | `string` | yes    | ParticipantID describing the source WebRTC session of the event. If the WebRTC session is publishing media, the participants own id is used |
| `media_session_type` | `enum`   | yes    | Either `"video"` or `"screen"`                                                                                                              |

### Target

| Field                | Type     | Required | Description                                                                                                                                        |
| -------------------- | -------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `target`             | `string` | yes      | ParticipantID describing the target WebRTC session of the command. If the WebRTC session is publishing media, the participants own id must be used |
| `media_session_type` | `enum`   | yes      | Either `"video"` or `"screen"`                                                                                                                     |

### MediaSessionState

This object represent the current mute status of a media session.

| Field   | Type   | Always | Description                |
| ------- | ------ | ------ | -------------------------- |
| `video` | `bool` | yes    | Video is enabled (unmuted) |
| `audio` | `bool` | yes    | Audio is enabled (unmuted) |

### ParticipantSpeakingState

| Field         | Type     | Required | Description                                                        |
| ------------- | -------- | -------- | ------------------------------------------------------------------ |
| `participant` | `string` | yes      | The id of the participant                                          |
| `is_speaking` | `bool`   | yes      | A flag indicating whether the participant is currently speaking    |
| `updated_at`  | `string` | yes      | Timestamp of the last change of `is_speaking` for this participant |

### TrickleCandidate

| Field           | Type     | Required | Description                                                                                              |
| --------------- | -------- | -------- | -------------------------------------------------------------------------------------------------------- |
| `sdpMLineIndex` | `int`    | yes      | [MDN web docs reference](https://developer.mozilla.org/en-US/docs/Web/API/RTCIceCandidate/sdpMLineIndex) |
| `candidate`     | `string` | yes      | [MDN web docs reference](https://developer.mozilla.org/en-US/docs/Web/API/RTCIceCandidate/candidate)     |

### ForceMuteState

| Field      | Type       | Required | Description                                                                                                 |
| ---------- | ---------- | -------- | ----------------------------------------------------------------------------------------------------------- |
| type       | `enum`     | yes      | Either `enabled` if the force-mute state is enabled, otherwise `disabled`                                   |
| allow_list | `string[]` | no       | If `type` is `enabled`, the `allow_list` contains the list of participants that are still allowed to unmute |
