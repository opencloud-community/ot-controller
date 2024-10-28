# Mail-Worker Protocol

This sections describes the schema and lists a few examples for messages to the
[OpenTalk SMTP Mailer](https://docs.opentalk.eu/admin/smtp-mailer).

## Schema

<!-- begin:fromfile:mail-worker-protocol/schema.json.md -->

```json
{
  "components": {
    "schemas": {
      "CallIn": {
        "type": "object",
        "required": [
          "sip_tel",
          "sip_id",
          "sip_password"
        ],
        "properties": {
          "sip_id": {
            "type": "string"
          },
          "sip_password": {
            "type": "string"
          },
          "sip_tel": {
            "type": "string"
          }
        },
        "example": {
          "sip_id": "1234567890",
          "sip_password": "9876543210",
          "sip_tel": "+99-1234567890"
        }
      },
      "Email": {
        "type": "string",
        "example": "alice.adams@example.com"
      },
      "Event": {
        "type": "object",
        "required": [
          "id",
          "name",
          "created_at",
          "description",
          "room",
          "revision",
          "streaming_targets"
        ],
        "properties": {
          "adhoc_retention_seconds": {
            "type": [
              "integer",
              "null"
            ],
            "format": "int64",
            "minimum": 0
          },
          "call_in": {
            "oneOf": [
              {
                "type": "null"
              },
              {
                "$ref": "#/components/schemas/CallIn"
              }
            ]
          },
          "created_at": {
            "$ref": "#/components/schemas/Time"
          },
          "description": {
            "$ref": "#/components/schemas/EventDescription"
          },
          "end_time": {
            "oneOf": [
              {
                "type": "null"
              },
              {
                "$ref": "#/components/schemas/Time"
              }
            ]
          },
          "id": {
            "type": "string",
            "format": "uuid"
          },
          "name": {
            "$ref": "#/components/schemas/EventTitle"
          },
          "revision": {
            "type": "integer",
            "format": "int32"
          },
          "room": {
            "$ref": "#/components/schemas/Room"
          },
          "rrule": {
            "type": [
              "string",
              "null"
            ]
          },
          "shared_folder": {
            "oneOf": [
              {
                "type": "null"
              },
              {
                "$ref": "#/components/schemas/SharedFolder"
              }
            ]
          },
          "start_time": {
            "oneOf": [
              {
                "type": "null"
              },
              {
                "$ref": "#/components/schemas/Time"
              }
            ]
          },
          "streaming_targets": {
            "type": "array",
            "items": {
              "$ref": "#/components/schemas/RoomStreamingTarget"
            }
          }
        },
        "example": {
          "adhoc_retention_seconds": null,
          "call_in": {
            "sip_id": "1234567890",
            "sip_password": "9876543210",
            "sip_tel": "+99-1234567890"
          },
          "created_at": {
            "time": "2024-07-05T17:02:42Z",
            "timezone": "Europe/Berlin"
          },
          "description": "The team's regular weekly meeting",
          "end_time": null,
          "id": "00000000-0000-0000-0000-0000abadcafe",
          "name": "Weekly teammeeting",
          "revision": 3,
          "room": {
            "id": "00000000-0000-0000-0000-0000abcdef99",
            "password": "v3rys3cr3t"
          },
          "rrule": null,
          "shared_folder": {
            "read": {
              "password": "v3rys3cr3t",
              "url": "https://cloud.example.com/shares/abc123"
            }
          },
          "start_time": null,
          "streaming_targets": []
        }
      },
      "EventDescription": {
        "type": "string",
        "description": "The description of an event",
        "examples": [
          "The Weekly Team Event"
        ],
        "maxLength": 4096
      },
      "EventException": {
        "type": "object",
        "required": [
          "exception_date",
          "kind"
        ],
        "properties": {
          "description": {
            "oneOf": [
              {
                "type": "null"
              },
              {
                "$ref": "#/components/schemas/EventDescription"
              }
            ]
          },
          "ends_at": {
            "oneOf": [
              {
                "type": "null"
              },
              {
                "$ref": "#/components/schemas/Time"
              }
            ]
          },
          "exception_date": {
            "$ref": "#/components/schemas/Time"
          },
          "is_all_day": {
            "type": [
              "boolean",
              "null"
            ]
          },
          "kind": {
            "$ref": "#/components/schemas/EventExceptionKind"
          },
          "starts_at": {
            "oneOf": [
              {
                "type": "null"
              },
              {
                "$ref": "#/components/schemas/Time"
              }
            ]
          },
          "title": {
            "oneOf": [
              {
                "type": "null"
              },
              {
                "$ref": "#/components/schemas/EventTitle"
              }
            ]
          }
        },
        "example": {
          "description": null,
          "ends_at": null,
          "exception_date": {
            "time": "2024-07-05T17:02:42Z",
            "timezone": "Europe/Berlin"
          },
          "is_all_day": null,
          "kind": "modified",
          "starts_at": null,
          "title": "Another weekly meeting"
        }
      },
      "EventExceptionKind": {
        "type": "string",
        "enum": [
          "modified",
          "canceled"
        ],
        "example": "modified"
      },
      "EventTitle": {
        "type": "string",
        "description": "The title of an event",
        "examples": [
          "Team Event"
        ],
        "maxLength": 255
      },
      "ExternalEventCancellation": {
        "type": "object",
        "required": [
          "invitee",
          "event",
          "inviter"
        ],
        "properties": {
          "event": {
            "$ref": "#/components/schemas/Event"
          },
          "invitee": {
            "$ref": "#/components/schemas/ExternalUser"
          },
          "inviter": {
            "$ref": "#/components/schemas/RegisteredUser"
          }
        },
        "example": {
          "event": {
            "adhoc_retention_seconds": null,
            "call_in": {
              "sip_id": "1234567890",
              "sip_password": "9876543210",
              "sip_tel": "+99-1234567890"
            },
            "created_at": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "description": "The team's regular weekly meeting",
            "end_time": null,
            "id": "00000000-0000-0000-0000-0000abadcafe",
            "name": "Weekly teammeeting",
            "revision": 3,
            "room": {
              "id": "00000000-0000-0000-0000-0000abcdef99",
              "password": "v3rys3cr3t"
            },
            "rrule": null,
            "shared_folder": {
              "read": {
                "password": "v3rys3cr3t",
                "url": "https://cloud.example.com/shares/abc123"
              }
            },
            "start_time": null,
            "streaming_targets": []
          },
          "invitee": {
            "email": "charlie.cooper@example.com"
          },
          "inviter": {
            "email": "dave.dunn@example.com",
            "first_name": "Dave",
            "language": "en",
            "last_name": "Dunn",
            "title": ""
          }
        }
      },
      "ExternalEventInvite": {
        "type": "object",
        "required": [
          "invitee",
          "event",
          "inviter",
          "invite_code"
        ],
        "properties": {
          "event": {
            "$ref": "#/components/schemas/Event"
          },
          "invite_code": {
            "type": "string"
          },
          "invitee": {
            "$ref": "#/components/schemas/ExternalUser"
          },
          "inviter": {
            "$ref": "#/components/schemas/RegisteredUser"
          }
        },
        "example": {
          "event": {
            "adhoc_retention_seconds": null,
            "call_in": {
              "sip_id": "1234567890",
              "sip_password": "9876543210",
              "sip_tel": "+99-1234567890"
            },
            "created_at": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "description": "The team's regular weekly meeting",
            "end_time": null,
            "id": "00000000-0000-0000-0000-0000abadcafe",
            "name": "Weekly teammeeting",
            "revision": 3,
            "room": {
              "id": "00000000-0000-0000-0000-0000abcdef99",
              "password": "v3rys3cr3t"
            },
            "rrule": null,
            "shared_folder": {
              "read": {
                "password": "v3rys3cr3t",
                "url": "https://cloud.example.com/shares/abc123"
              }
            },
            "start_time": null,
            "streaming_targets": []
          },
          "invite_code": "5b5f9cf0-86a8-4e5a-bfac-b05c34c8a20b",
          "invitee": {
            "email": "charlie.cooper@example.com"
          },
          "inviter": {
            "email": "dave.dunn@example.com",
            "first_name": "Dave",
            "language": "en",
            "last_name": "Dunn",
            "title": ""
          }
        }
      },
      "ExternalEventUninvite": {
        "type": "object",
        "required": [
          "invitee",
          "event",
          "inviter"
        ],
        "properties": {
          "event": {
            "$ref": "#/components/schemas/Event"
          },
          "invitee": {
            "$ref": "#/components/schemas/ExternalUser"
          },
          "inviter": {
            "$ref": "#/components/schemas/RegisteredUser"
          }
        },
        "example": {
          "event": {
            "adhoc_retention_seconds": null,
            "call_in": {
              "sip_id": "1234567890",
              "sip_password": "9876543210",
              "sip_tel": "+99-1234567890"
            },
            "created_at": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "description": "The team's regular weekly meeting",
            "end_time": null,
            "id": "00000000-0000-0000-0000-0000abadcafe",
            "name": "Weekly teammeeting",
            "revision": 3,
            "room": {
              "id": "00000000-0000-0000-0000-0000abcdef99",
              "password": "v3rys3cr3t"
            },
            "rrule": null,
            "shared_folder": {
              "read": {
                "password": "v3rys3cr3t",
                "url": "https://cloud.example.com/shares/abc123"
              }
            },
            "start_time": null,
            "streaming_targets": []
          },
          "invitee": {
            "email": "charlie.cooper@example.com"
          },
          "inviter": {
            "email": "dave.dunn@example.com",
            "first_name": "Dave",
            "language": "en",
            "last_name": "Dunn",
            "title": ""
          }
        }
      },
      "ExternalEventUpdate": {
        "type": "object",
        "required": [
          "invitee",
          "event",
          "inviter",
          "invite_code"
        ],
        "properties": {
          "event": {
            "$ref": "#/components/schemas/Event"
          },
          "event_exception": {
            "oneOf": [
              {
                "type": "null"
              },
              {
                "$ref": "#/components/schemas/EventException"
              }
            ]
          },
          "invite_code": {
            "type": "string"
          },
          "invitee": {
            "$ref": "#/components/schemas/ExternalUser"
          },
          "inviter": {
            "$ref": "#/components/schemas/RegisteredUser"
          }
        },
        "example": {
          "event": {
            "adhoc_retention_seconds": null,
            "call_in": {
              "sip_id": "1234567890",
              "sip_password": "9876543210",
              "sip_tel": "+99-1234567890"
            },
            "created_at": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "description": "The team's regular weekly meeting",
            "end_time": null,
            "id": "00000000-0000-0000-0000-0000abadcafe",
            "name": "Weekly teammeeting",
            "revision": 3,
            "room": {
              "id": "00000000-0000-0000-0000-0000abcdef99",
              "password": "v3rys3cr3t"
            },
            "rrule": null,
            "shared_folder": {
              "read": {
                "password": "v3rys3cr3t",
                "url": "https://cloud.example.com/shares/abc123"
              }
            },
            "start_time": null,
            "streaming_targets": []
          },
          "event_exception": {
            "description": null,
            "ends_at": null,
            "exception_date": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "is_all_day": null,
            "kind": "modified",
            "starts_at": null,
            "title": "Another weekly meeting"
          },
          "invite_code": "5b5f9cf0-86a8-4e5a-bfac-b05c34c8a20b",
          "invitee": {
            "email": "charlie.cooper@example.com"
          },
          "inviter": {
            "email": "dave.dunn@example.com",
            "first_name": "Dave",
            "language": "en",
            "last_name": "Dunn",
            "title": ""
          }
        }
      },
      "ExternalUser": {
        "type": "object",
        "required": [
          "email"
        ],
        "properties": {
          "email": {
            "$ref": "#/components/schemas/Email"
          }
        },
        "example": {
          "email": "charlie.cooper@example.com"
        }
      },
      "Language": {
        "type": "string",
        "description": "A language identifier",
        "examples": [
          "de"
        ],
        "maxLength": 35
      },
      "MailTask": {
        "oneOf": [
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/v1.Message"
              },
              {
                "type": "object",
                "required": [
                  "version"
                ],
                "properties": {
                  "version": {
                    "type": "string",
                    "enum": [
                      "1"
                    ]
                  }
                }
              }
            ]
          }
        ],
        "description": "Versioned Mail Task Protocol"
      },
      "RegisteredEventCancellation": {
        "type": "object",
        "required": [
          "invitee",
          "event",
          "inviter"
        ],
        "properties": {
          "event": {
            "$ref": "#/components/schemas/Event"
          },
          "invitee": {
            "$ref": "#/components/schemas/RegisteredUser"
          },
          "inviter": {
            "$ref": "#/components/schemas/RegisteredUser"
          }
        },
        "example": {
          "event": {
            "adhoc_retention_seconds": null,
            "call_in": {
              "sip_id": "1234567890",
              "sip_password": "9876543210",
              "sip_tel": "+99-1234567890"
            },
            "created_at": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "description": "The team's regular weekly meeting",
            "end_time": null,
            "id": "00000000-0000-0000-0000-0000abadcafe",
            "name": "Weekly teammeeting",
            "revision": 3,
            "room": {
              "id": "00000000-0000-0000-0000-0000abcdef99",
              "password": "v3rys3cr3t"
            },
            "rrule": null,
            "shared_folder": {
              "read": {
                "password": "v3rys3cr3t",
                "url": "https://cloud.example.com/shares/abc123"
              }
            },
            "start_time": null,
            "streaming_targets": []
          },
          "invitee": {
            "email": "alice.adams@example.com",
            "first_name": "Alice",
            "language": "en",
            "last_name": "Adams",
            "title": "Dr."
          },
          "inviter": {
            "email": "dave.dunn@example.com",
            "first_name": "Dave",
            "language": "en",
            "last_name": "Dunn",
            "title": ""
          }
        }
      },
      "RegisteredEventInvite": {
        "type": "object",
        "required": [
          "invitee",
          "event",
          "inviter"
        ],
        "properties": {
          "event": {
            "$ref": "#/components/schemas/Event"
          },
          "invitee": {
            "$ref": "#/components/schemas/RegisteredUser"
          },
          "inviter": {
            "$ref": "#/components/schemas/RegisteredUser"
          }
        },
        "example": {
          "event": {
            "adhoc_retention_seconds": null,
            "call_in": {
              "sip_id": "1234567890",
              "sip_password": "9876543210",
              "sip_tel": "+99-1234567890"
            },
            "created_at": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "description": "The team's regular weekly meeting",
            "end_time": null,
            "id": "00000000-0000-0000-0000-0000abadcafe",
            "name": "Weekly teammeeting",
            "revision": 3,
            "room": {
              "id": "00000000-0000-0000-0000-0000abcdef99",
              "password": "v3rys3cr3t"
            },
            "rrule": null,
            "shared_folder": {
              "read": {
                "password": "v3rys3cr3t",
                "url": "https://cloud.example.com/shares/abc123"
              }
            },
            "start_time": null,
            "streaming_targets": []
          },
          "invitee": {
            "email": "alice.adams@example.com",
            "first_name": "Alice",
            "language": "en",
            "last_name": "Adams",
            "title": "Dr."
          },
          "inviter": {
            "email": "dave.dunn@example.com",
            "first_name": "Dave",
            "language": "en",
            "last_name": "Dunn",
            "title": ""
          }
        }
      },
      "RegisteredEventUninvite": {
        "type": "object",
        "required": [
          "invitee",
          "event",
          "inviter"
        ],
        "properties": {
          "event": {
            "$ref": "#/components/schemas/Event"
          },
          "invitee": {
            "$ref": "#/components/schemas/RegisteredUser"
          },
          "inviter": {
            "$ref": "#/components/schemas/RegisteredUser"
          }
        },
        "example": {
          "event": {
            "adhoc_retention_seconds": null,
            "call_in": {
              "sip_id": "1234567890",
              "sip_password": "9876543210",
              "sip_tel": "+99-1234567890"
            },
            "created_at": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "description": "The team's regular weekly meeting",
            "end_time": null,
            "id": "00000000-0000-0000-0000-0000abadcafe",
            "name": "Weekly teammeeting",
            "revision": 3,
            "room": {
              "id": "00000000-0000-0000-0000-0000abcdef99",
              "password": "v3rys3cr3t"
            },
            "rrule": null,
            "shared_folder": {
              "read": {
                "password": "v3rys3cr3t",
                "url": "https://cloud.example.com/shares/abc123"
              }
            },
            "start_time": null,
            "streaming_targets": []
          },
          "invitee": {
            "email": "alice.adams@example.com",
            "first_name": "Alice",
            "language": "en",
            "last_name": "Adams",
            "title": "Dr."
          },
          "inviter": {
            "email": "dave.dunn@example.com",
            "first_name": "Dave",
            "language": "en",
            "last_name": "Dunn",
            "title": ""
          }
        }
      },
      "RegisteredEventUpdate": {
        "type": "object",
        "required": [
          "invitee",
          "event",
          "inviter"
        ],
        "properties": {
          "event": {
            "$ref": "#/components/schemas/Event"
          },
          "event_exception": {
            "oneOf": [
              {
                "type": "null"
              },
              {
                "$ref": "#/components/schemas/EventException"
              }
            ]
          },
          "invitee": {
            "$ref": "#/components/schemas/RegisteredUser"
          },
          "inviter": {
            "$ref": "#/components/schemas/RegisteredUser"
          }
        },
        "example": {
          "event": {
            "adhoc_retention_seconds": null,
            "call_in": {
              "sip_id": "1234567890",
              "sip_password": "9876543210",
              "sip_tel": "+99-1234567890"
            },
            "created_at": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "description": "The team's regular weekly meeting",
            "end_time": null,
            "id": "00000000-0000-0000-0000-0000abadcafe",
            "name": "Weekly teammeeting",
            "revision": 3,
            "room": {
              "id": "00000000-0000-0000-0000-0000abcdef99",
              "password": "v3rys3cr3t"
            },
            "rrule": null,
            "shared_folder": {
              "read": {
                "password": "v3rys3cr3t",
                "url": "https://cloud.example.com/shares/abc123"
              }
            },
            "start_time": null,
            "streaming_targets": []
          },
          "event_exception": {
            "description": null,
            "ends_at": null,
            "exception_date": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "is_all_day": null,
            "kind": "modified",
            "starts_at": null,
            "title": "Another weekly meeting"
          },
          "invitee": {
            "email": "alice.adams@example.com",
            "first_name": "Alice",
            "language": "en",
            "last_name": "Adams",
            "title": "Dr."
          },
          "inviter": {
            "email": "dave.dunn@example.com",
            "first_name": "Dave",
            "language": "en",
            "last_name": "Dunn",
            "title": ""
          }
        }
      },
      "RegisteredUser": {
        "type": "object",
        "required": [
          "email",
          "title",
          "first_name",
          "last_name",
          "language"
        ],
        "properties": {
          "email": {
            "$ref": "#/components/schemas/Email"
          },
          "first_name": {
            "type": "string"
          },
          "language": {
            "$ref": "#/components/schemas/Language"
          },
          "last_name": {
            "type": "string"
          },
          "title": {
            "$ref": "#/components/schemas/UserTitle"
          }
        },
        "example": {
          "email": "alice.adams@example.com",
          "first_name": "Alice",
          "language": "en",
          "last_name": "Adams",
          "title": "Dr."
        }
      },
      "Room": {
        "type": "object",
        "required": [
          "id"
        ],
        "properties": {
          "id": {
            "type": "string",
            "format": "uuid"
          },
          "password": {
            "oneOf": [
              {
                "type": "null"
              },
              {
                "$ref": "#/components/schemas/RoomPassword"
              }
            ]
          }
        },
        "example": {
          "id": "00000000-0000-0000-0000-0000abcdef99",
          "password": "v3rys3cr3t"
        }
      },
      "RoomPassword": {
        "type": "string",
        "description": "A room password",
        "examples": [
          "v3rys3cr3t"
        ],
        "maxLength": 255,
        "minLength": 1
      },
      "RoomStreamingTarget": {
        "allOf": [
          {
            "$ref": "#/components/schemas/StreamingTarget",
            "description": "The streaming target"
          },
          {
            "type": "object",
            "required": [
              "id"
            ],
            "properties": {
              "id": {
                "$ref": "#/components/schemas/StreamingTargetId",
                "description": "The streaming target id"
              }
            }
          }
        ],
        "description": "A streaming target which is specific for a Room",
        "example": {
          "id": "00000000-0000-0000-0000-000043434343",
          "kind": "custom",
          "name": "Example Stream",
          "public_url": "https://streaming.example.com/livestream123",
          "streaming_endpoint": "https://ingress.streaming.example.com/",
          "streaming_key": "aabbccddeeff"
        }
      },
      "SharedFolder": {
        "type": "object",
        "description": "Information about a shared folder containing\nread and optional write access",
        "required": [
          "read"
        ],
        "properties": {
          "read": {
            "$ref": "#/components/schemas/SharedFolderAccess",
            "description": "Read access information for the shared folder"
          },
          "read_write": {
            "$ref": "#/components/schemas/SharedFolderAccess",
            "description": "Read-write access information for the shared folder"
          }
        },
        "example": {
          "read": {
            "password": "v3rys3cr3t",
            "url": "https://cloud.example.com/shares/abc123"
          }
        }
      },
      "SharedFolderAccess": {
        "type": "object",
        "description": "Information required to access a shared folder",
        "required": [
          "url",
          "password"
        ],
        "properties": {
          "password": {
            "type": "string",
            "description": "Password required to access the shared folder"
          },
          "url": {
            "type": "string",
            "description": "Shared folder URL"
          }
        },
        "example": {
          "password": "v3rys3cr3t",
          "url": "https://cloud.example.com/shares/abc123"
        }
      },
      "StreamingKey": {
        "type": "string",
        "description": "The secret key of a streaming target",
        "example": "aabbccddeeff"
      },
      "StreamingTarget": {
        "allOf": [
          {
            "$ref": "#/components/schemas/StreamingTargetKind",
            "description": "The kind of the streaming target"
          },
          {
            "type": "object",
            "required": [
              "name"
            ],
            "properties": {
              "name": {
                "type": "string",
                "description": "The name of the streaming target"
              }
            }
          }
        ],
        "description": "A streaming target",
        "example": {
          "kind": "custom",
          "name": "Example Stream",
          "public_url": "https://streaming.example.com/livestream123",
          "streaming_endpoint": "https://ingress.streaming.example.com/",
          "streaming_key": "aabbccddeeff"
        }
      },
      "StreamingTargetId": {
        "type": "string",
        "format": "uuid",
        "description": "ID of a streaming target",
        "example": "00000000-0000-0000-0000-000043434343"
      },
      "StreamingTargetKind": {
        "oneOf": [
          {
            "type": "object",
            "description": "The \"custom\" kind",
            "required": [
              "streaming_endpoint",
              "streaming_key",
              "public_url",
              "kind"
            ],
            "properties": {
              "kind": {
                "type": "string",
                "enum": [
                  "custom"
                ]
              },
              "public_url": {
                "type": "string",
                "format": "uri",
                "description": "The url from which the stream can be accessed"
              },
              "streaming_endpoint": {
                "type": "string",
                "format": "uri",
                "description": "The endpoint url of the streaming target"
              },
              "streaming_key": {
                "$ref": "#/components/schemas/StreamingKey",
                "description": "The streaming key"
              }
            }
          }
        ],
        "description": "A streaming target kind",
        "example": {
          "kind": "custom",
          "public_url": "https://streaming.example.com/livestream123",
          "streaming_endpoint": "https://ingress.streaming.example.com/",
          "streaming_key": "aabbccddeeff"
        }
      },
      "Time": {
        "type": "object",
        "required": [
          "time",
          "timezone"
        ],
        "properties": {
          "time": {
            "type": "string",
            "format": "date-time"
          },
          "timezone": {
            "type": "string"
          }
        }
      },
      "UnregisteredEventCancellation": {
        "type": "object",
        "required": [
          "invitee",
          "event",
          "inviter"
        ],
        "properties": {
          "event": {
            "$ref": "#/components/schemas/Event"
          },
          "invitee": {
            "$ref": "#/components/schemas/UnregisteredUser"
          },
          "inviter": {
            "$ref": "#/components/schemas/RegisteredUser"
          }
        },
        "example": {
          "event": {
            "adhoc_retention_seconds": null,
            "call_in": {
              "sip_id": "1234567890",
              "sip_password": "9876543210",
              "sip_tel": "+99-1234567890"
            },
            "created_at": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "description": "The team's regular weekly meeting",
            "end_time": null,
            "id": "00000000-0000-0000-0000-0000abadcafe",
            "name": "Weekly teammeeting",
            "revision": 3,
            "room": {
              "id": "00000000-0000-0000-0000-0000abcdef99",
              "password": "v3rys3cr3t"
            },
            "rrule": null,
            "shared_folder": {
              "read": {
                "password": "v3rys3cr3t",
                "url": "https://cloud.example.com/shares/abc123"
              }
            },
            "start_time": null,
            "streaming_targets": []
          },
          "invitee": {
            "email": "bob.burton@example.com",
            "first_name": "Bob",
            "last_name": "Burton"
          },
          "inviter": {
            "email": "dave.dunn@example.com",
            "first_name": "Dave",
            "language": "en",
            "last_name": "Dunn",
            "title": ""
          }
        }
      },
      "UnregisteredEventInvite": {
        "type": "object",
        "required": [
          "invitee",
          "event",
          "inviter"
        ],
        "properties": {
          "event": {
            "$ref": "#/components/schemas/Event"
          },
          "invitee": {
            "$ref": "#/components/schemas/UnregisteredUser"
          },
          "inviter": {
            "$ref": "#/components/schemas/RegisteredUser"
          }
        },
        "example": {
          "event": {
            "adhoc_retention_seconds": null,
            "call_in": {
              "sip_id": "1234567890",
              "sip_password": "9876543210",
              "sip_tel": "+99-1234567890"
            },
            "created_at": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "description": "The team's regular weekly meeting",
            "end_time": null,
            "id": "00000000-0000-0000-0000-0000abadcafe",
            "name": "Weekly teammeeting",
            "revision": 3,
            "room": {
              "id": "00000000-0000-0000-0000-0000abcdef99",
              "password": "v3rys3cr3t"
            },
            "rrule": null,
            "shared_folder": {
              "read": {
                "password": "v3rys3cr3t",
                "url": "https://cloud.example.com/shares/abc123"
              }
            },
            "start_time": null,
            "streaming_targets": []
          },
          "invitee": {
            "email": "bob.burton@example.com",
            "first_name": "Bob",
            "last_name": "Burton"
          },
          "inviter": {
            "email": "dave.dunn@example.com",
            "first_name": "Dave",
            "language": "en",
            "last_name": "Dunn",
            "title": ""
          }
        }
      },
      "UnregisteredEventUninvite": {
        "type": "object",
        "required": [
          "invitee",
          "event",
          "inviter"
        ],
        "properties": {
          "event": {
            "$ref": "#/components/schemas/Event"
          },
          "invitee": {
            "$ref": "#/components/schemas/UnregisteredUser"
          },
          "inviter": {
            "$ref": "#/components/schemas/RegisteredUser"
          }
        },
        "example": {
          "event": {
            "adhoc_retention_seconds": null,
            "call_in": {
              "sip_id": "1234567890",
              "sip_password": "9876543210",
              "sip_tel": "+99-1234567890"
            },
            "created_at": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "description": "The team's regular weekly meeting",
            "end_time": null,
            "id": "00000000-0000-0000-0000-0000abadcafe",
            "name": "Weekly teammeeting",
            "revision": 3,
            "room": {
              "id": "00000000-0000-0000-0000-0000abcdef99",
              "password": "v3rys3cr3t"
            },
            "rrule": null,
            "shared_folder": {
              "read": {
                "password": "v3rys3cr3t",
                "url": "https://cloud.example.com/shares/abc123"
              }
            },
            "start_time": null,
            "streaming_targets": []
          },
          "invitee": {
            "email": "bob.burton@example.com",
            "first_name": "Bob",
            "last_name": "Burton"
          },
          "inviter": {
            "email": "dave.dunn@example.com",
            "first_name": "Dave",
            "language": "en",
            "last_name": "Dunn",
            "title": ""
          }
        }
      },
      "UnregisteredEventUpdate": {
        "type": "object",
        "required": [
          "invitee",
          "event",
          "inviter"
        ],
        "properties": {
          "event": {
            "$ref": "#/components/schemas/Event"
          },
          "event_exception": {
            "oneOf": [
              {
                "type": "null"
              },
              {
                "$ref": "#/components/schemas/EventException"
              }
            ]
          },
          "invitee": {
            "$ref": "#/components/schemas/UnregisteredUser"
          },
          "inviter": {
            "$ref": "#/components/schemas/RegisteredUser"
          }
        },
        "example": {
          "event": {
            "adhoc_retention_seconds": null,
            "call_in": {
              "sip_id": "1234567890",
              "sip_password": "9876543210",
              "sip_tel": "+99-1234567890"
            },
            "created_at": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "description": "The team's regular weekly meeting",
            "end_time": null,
            "id": "00000000-0000-0000-0000-0000abadcafe",
            "name": "Weekly teammeeting",
            "revision": 3,
            "room": {
              "id": "00000000-0000-0000-0000-0000abcdef99",
              "password": "v3rys3cr3t"
            },
            "rrule": null,
            "shared_folder": {
              "read": {
                "password": "v3rys3cr3t",
                "url": "https://cloud.example.com/shares/abc123"
              }
            },
            "start_time": null,
            "streaming_targets": []
          },
          "event_exception": {
            "description": null,
            "ends_at": null,
            "exception_date": {
              "time": "2024-07-05T17:02:42Z",
              "timezone": "Europe/Berlin"
            },
            "is_all_day": null,
            "kind": "modified",
            "starts_at": null,
            "title": "Another weekly meeting"
          },
          "invitee": {
            "email": "bob.burton@example.com",
            "first_name": "Bob",
            "last_name": "Burton"
          },
          "inviter": {
            "email": "dave.dunn@example.com",
            "first_name": "Dave",
            "language": "en",
            "last_name": "Dunn",
            "title": ""
          }
        }
      },
      "UnregisteredUser": {
        "type": "object",
        "required": [
          "email",
          "first_name",
          "last_name"
        ],
        "properties": {
          "email": {
            "$ref": "#/components/schemas/Email"
          },
          "first_name": {
            "type": "string"
          },
          "last_name": {
            "type": "string"
          }
        },
        "example": {
          "email": "bob.burton@example.com",
          "first_name": "Bob",
          "last_name": "Burton"
        }
      },
      "UserTitle": {
        "type": "string",
        "description": "The title of a user",
        "examples": [
          "M.Sc."
        ],
        "maxLength": 255
      },
      "v1.Message": {
        "oneOf": [
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/RegisteredEventInvite"
              },
              {
                "type": "object",
                "required": [
                  "message"
                ],
                "properties": {
                  "message": {
                    "type": "string",
                    "enum": [
                      "registered_event_invite"
                    ]
                  }
                }
              }
            ]
          },
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/UnregisteredEventInvite"
              },
              {
                "type": "object",
                "required": [
                  "message"
                ],
                "properties": {
                  "message": {
                    "type": "string",
                    "enum": [
                      "unregistered_event_invite"
                    ]
                  }
                }
              }
            ]
          },
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/ExternalEventInvite"
              },
              {
                "type": "object",
                "required": [
                  "message"
                ],
                "properties": {
                  "message": {
                    "type": "string",
                    "enum": [
                      "external_event_invite"
                    ]
                  }
                }
              }
            ]
          },
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/RegisteredEventUpdate"
              },
              {
                "type": "object",
                "required": [
                  "message"
                ],
                "properties": {
                  "message": {
                    "type": "string",
                    "enum": [
                      "registered_event_update"
                    ]
                  }
                }
              }
            ]
          },
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/UnregisteredEventUpdate"
              },
              {
                "type": "object",
                "required": [
                  "message"
                ],
                "properties": {
                  "message": {
                    "type": "string",
                    "enum": [
                      "unregistered_event_update"
                    ]
                  }
                }
              }
            ]
          },
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/ExternalEventUpdate"
              },
              {
                "type": "object",
                "required": [
                  "message"
                ],
                "properties": {
                  "message": {
                    "type": "string",
                    "enum": [
                      "external_event_update"
                    ]
                  }
                }
              }
            ]
          },
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/RegisteredEventCancellation"
              },
              {
                "type": "object",
                "required": [
                  "message"
                ],
                "properties": {
                  "message": {
                    "type": "string",
                    "enum": [
                      "registered_event_cancellation"
                    ]
                  }
                }
              }
            ]
          },
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/UnregisteredEventCancellation"
              },
              {
                "type": "object",
                "required": [
                  "message"
                ],
                "properties": {
                  "message": {
                    "type": "string",
                    "enum": [
                      "unregistered_event_cancellation"
                    ]
                  }
                }
              }
            ]
          },
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/ExternalEventCancellation"
              },
              {
                "type": "object",
                "required": [
                  "message"
                ],
                "properties": {
                  "message": {
                    "type": "string",
                    "enum": [
                      "external_event_cancellation"
                    ]
                  }
                }
              }
            ]
          },
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/RegisteredEventUninvite"
              },
              {
                "type": "object",
                "required": [
                  "message"
                ],
                "properties": {
                  "message": {
                    "type": "string",
                    "enum": [
                      "registered_event_uninvite"
                    ]
                  }
                }
              }
            ]
          },
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/UnregisteredEventUninvite"
              },
              {
                "type": "object",
                "required": [
                  "message"
                ],
                "properties": {
                  "message": {
                    "type": "string",
                    "enum": [
                      "unregistered_event_uninvite"
                    ]
                  }
                }
              }
            ]
          },
          {
            "allOf": [
              {
                "$ref": "#/components/schemas/ExternalEventUninvite"
              },
              {
                "type": "object",
                "required": [
                  "message"
                ],
                "properties": {
                  "message": {
                    "type": "string",
                    "enum": [
                      "external_event_uninvite"
                    ]
                  }
                }
              }
            ]
          }
        ],
        "description": "The different kinds of MailTasks that are currently supported"
      }
    }
  }
```

<!-- end:fromfile:mail-worker-protocol/schema.json.md -->

## Examples

### Registered user notifications

#### Invite

<!-- begin:fromfile:mail-worker-protocol/registered_event_invite.json.md -->

```json
{
  "version": "1",
  "message": "registered_event_invite",
  "invitee": {
    "email": "alice.adams@example.com",
    "title": "Dr.",
    "first_name": "Alice",
    "last_name": "Adams",
    "language": "en"
  },
  "event": {
    "id": "00000000-0000-0000-0000-0000abadcafe",
    "name": "Weekly teammeeting",
    "created_at": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "start_time": null,
    "end_time": null,
    "rrule": null,
    "description": "The team's regular weekly meeting",
    "room": {
      "id": "00000000-0000-0000-0000-0000abcdef99",
      "password": "v3rys3cr3t"
    },
    "call_in": {
      "sip_tel": "+99-1234567890",
      "sip_id": "1234567890",
      "sip_password": "9876543210"
    },
    "revision": 3,
    "shared_folder": {
      "read": {
        "url": "https://cloud.example.com/shares/abc123",
        "password": "v3rys3cr3t"
      }
    },
    "adhoc_retention_seconds": null,
    "streaming_targets": []
  },
  "inviter": {
    "email": "dave.dunn@example.com",
    "title": "",
    "first_name": "Dave",
    "last_name": "Dunn",
    "language": "en"
  }
```

<!-- end:fromfile:mail-worker-protocol/registered_event_invite.json.md -->

#### Update

<!-- begin:fromfile:mail-worker-protocol/registered_event_update.json.md -->

```json
{
  "version": "1",
  "message": "registered_event_update",
  "invitee": {
    "email": "alice.adams@example.com",
    "title": "Dr.",
    "first_name": "Alice",
    "last_name": "Adams",
    "language": "en"
  },
  "event": {
    "id": "00000000-0000-0000-0000-0000abadcafe",
    "name": "Weekly teammeeting",
    "created_at": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "start_time": null,
    "end_time": null,
    "rrule": null,
    "description": "The team's regular weekly meeting",
    "room": {
      "id": "00000000-0000-0000-0000-0000abcdef99",
      "password": "v3rys3cr3t"
    },
    "call_in": {
      "sip_tel": "+99-1234567890",
      "sip_id": "1234567890",
      "sip_password": "9876543210"
    },
    "revision": 3,
    "shared_folder": {
      "read": {
        "url": "https://cloud.example.com/shares/abc123",
        "password": "v3rys3cr3t"
      }
    },
    "adhoc_retention_seconds": null,
    "streaming_targets": []
  },
  "event_exception": {
    "exception_date": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "kind": "modified",
    "title": "Another weekly meeting",
    "description": null,
    "is_all_day": null,
    "starts_at": null,
    "ends_at": null
  },
  "inviter": {
    "email": "dave.dunn@example.com",
    "title": "",
    "first_name": "Dave",
    "last_name": "Dunn",
    "language": "en"
  }
```

<!-- end:fromfile:mail-worker-protocol/registered_event_update.json.md -->

#### Uninvite

<!-- begin:fromfile:mail-worker-protocol/registered_event_uninvite.json.md -->

```json
{
  "version": "1",
  "message": "registered_event_uninvite",
  "invitee": {
    "email": "alice.adams@example.com",
    "title": "Dr.",
    "first_name": "Alice",
    "last_name": "Adams",
    "language": "en"
  },
  "event": {
    "id": "00000000-0000-0000-0000-0000abadcafe",
    "name": "Weekly teammeeting",
    "created_at": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "start_time": null,
    "end_time": null,
    "rrule": null,
    "description": "The team's regular weekly meeting",
    "room": {
      "id": "00000000-0000-0000-0000-0000abcdef99",
      "password": "v3rys3cr3t"
    },
    "call_in": {
      "sip_tel": "+99-1234567890",
      "sip_id": "1234567890",
      "sip_password": "9876543210"
    },
    "revision": 3,
    "shared_folder": {
      "read": {
        "url": "https://cloud.example.com/shares/abc123",
        "password": "v3rys3cr3t"
      }
    },
    "adhoc_retention_seconds": null,
    "streaming_targets": []
  },
  "inviter": {
    "email": "dave.dunn@example.com",
    "title": "",
    "first_name": "Dave",
    "last_name": "Dunn",
    "language": "en"
  }
```

<!-- end:fromfile:mail-worker-protocol/registered_event_uninvite.json.md -->

#### Cancellation

<!-- begin:fromfile:mail-worker-protocol/registered_event_cancellation.json.md -->

```json
{
  "version": "1",
  "message": "registered_event_cancellation",
  "invitee": {
    "email": "alice.adams@example.com",
    "title": "Dr.",
    "first_name": "Alice",
    "last_name": "Adams",
    "language": "en"
  },
  "event": {
    "id": "00000000-0000-0000-0000-0000abadcafe",
    "name": "Weekly teammeeting",
    "created_at": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "start_time": null,
    "end_time": null,
    "rrule": null,
    "description": "The team's regular weekly meeting",
    "room": {
      "id": "00000000-0000-0000-0000-0000abcdef99",
      "password": "v3rys3cr3t"
    },
    "call_in": {
      "sip_tel": "+99-1234567890",
      "sip_id": "1234567890",
      "sip_password": "9876543210"
    },
    "revision": 3,
    "shared_folder": {
      "read": {
        "url": "https://cloud.example.com/shares/abc123",
        "password": "v3rys3cr3t"
      }
    },
    "adhoc_retention_seconds": null,
    "streaming_targets": []
  },
  "inviter": {
    "email": "dave.dunn@example.com",
    "title": "",
    "first_name": "Dave",
    "last_name": "Dunn",
    "language": "en"
  }
```

<!-- end:fromfile:mail-worker-protocol/registered_event_cancellation.json.md -->

### Unregistered user notifications

#### Invite

<!-- begin:fromfile:mail-worker-protocol/unregistered_event_invite.json.md -->

```json
{
  "version": "1",
  "message": "unregistered_event_invite",
  "invitee": {
    "email": "bob.burton@example.com",
    "first_name": "Bob",
    "last_name": "Burton"
  },
  "event": {
    "id": "00000000-0000-0000-0000-0000abadcafe",
    "name": "Weekly teammeeting",
    "created_at": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "start_time": null,
    "end_time": null,
    "rrule": null,
    "description": "The team's regular weekly meeting",
    "room": {
      "id": "00000000-0000-0000-0000-0000abcdef99",
      "password": "v3rys3cr3t"
    },
    "call_in": {
      "sip_tel": "+99-1234567890",
      "sip_id": "1234567890",
      "sip_password": "9876543210"
    },
    "revision": 3,
    "shared_folder": {
      "read": {
        "url": "https://cloud.example.com/shares/abc123",
        "password": "v3rys3cr3t"
      }
    },
    "adhoc_retention_seconds": null,
    "streaming_targets": []
  },
  "inviter": {
    "email": "dave.dunn@example.com",
    "title": "",
    "first_name": "Dave",
    "last_name": "Dunn",
    "language": "en"
  }
```

<!-- end:fromfile:mail-worker-protocol/unregistered_event_invite.json.md -->

#### Update

<!-- begin:fromfile:mail-worker-protocol/unregistered_event_update.json.md -->

```json
{
  "version": "1",
  "message": "unregistered_event_update",
  "invitee": {
    "email": "bob.burton@example.com",
    "first_name": "Bob",
    "last_name": "Burton"
  },
  "event": {
    "id": "00000000-0000-0000-0000-0000abadcafe",
    "name": "Weekly teammeeting",
    "created_at": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "start_time": null,
    "end_time": null,
    "rrule": null,
    "description": "The team's regular weekly meeting",
    "room": {
      "id": "00000000-0000-0000-0000-0000abcdef99",
      "password": "v3rys3cr3t"
    },
    "call_in": {
      "sip_tel": "+99-1234567890",
      "sip_id": "1234567890",
      "sip_password": "9876543210"
    },
    "revision": 3,
    "shared_folder": {
      "read": {
        "url": "https://cloud.example.com/shares/abc123",
        "password": "v3rys3cr3t"
      }
    },
    "adhoc_retention_seconds": null,
    "streaming_targets": []
  },
  "event_exception": {
    "exception_date": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "kind": "modified",
    "title": "Another weekly meeting",
    "description": null,
    "is_all_day": null,
    "starts_at": null,
    "ends_at": null
  },
  "inviter": {
    "email": "dave.dunn@example.com",
    "title": "",
    "first_name": "Dave",
    "last_name": "Dunn",
    "language": "en"
  }
```

<!-- end:fromfile:mail-worker-protocol/unregistered_event_update.json.md -->

#### Uninvite

<!-- begin:fromfile:mail-worker-protocol/unregistered_event_uninvite.json.md -->

```json
{
  "version": "1",
  "message": "unregistered_event_uninvite",
  "invitee": {
    "email": "bob.burton@example.com",
    "first_name": "Bob",
    "last_name": "Burton"
  },
  "event": {
    "id": "00000000-0000-0000-0000-0000abadcafe",
    "name": "Weekly teammeeting",
    "created_at": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "start_time": null,
    "end_time": null,
    "rrule": null,
    "description": "The team's regular weekly meeting",
    "room": {
      "id": "00000000-0000-0000-0000-0000abcdef99",
      "password": "v3rys3cr3t"
    },
    "call_in": {
      "sip_tel": "+99-1234567890",
      "sip_id": "1234567890",
      "sip_password": "9876543210"
    },
    "revision": 3,
    "shared_folder": {
      "read": {
        "url": "https://cloud.example.com/shares/abc123",
        "password": "v3rys3cr3t"
      }
    },
    "adhoc_retention_seconds": null,
    "streaming_targets": []
  },
  "inviter": {
    "email": "dave.dunn@example.com",
    "title": "",
    "first_name": "Dave",
    "last_name": "Dunn",
    "language": "en"
  }
```

<!-- end:fromfile:mail-worker-protocol/unregistered_event_uninvite.json.md -->

#### Cancellation

<!-- begin:fromfile:mail-worker-protocol/unregistered_event_cancellation.json.md -->

```json
{
  "version": "1",
  "message": "unregistered_event_cancellation",
  "invitee": {
    "email": "bob.burton@example.com",
    "first_name": "Bob",
    "last_name": "Burton"
  },
  "event": {
    "id": "00000000-0000-0000-0000-0000abadcafe",
    "name": "Weekly teammeeting",
    "created_at": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "start_time": null,
    "end_time": null,
    "rrule": null,
    "description": "The team's regular weekly meeting",
    "room": {
      "id": "00000000-0000-0000-0000-0000abcdef99",
      "password": "v3rys3cr3t"
    },
    "call_in": {
      "sip_tel": "+99-1234567890",
      "sip_id": "1234567890",
      "sip_password": "9876543210"
    },
    "revision": 3,
    "shared_folder": {
      "read": {
        "url": "https://cloud.example.com/shares/abc123",
        "password": "v3rys3cr3t"
      }
    },
    "adhoc_retention_seconds": null,
    "streaming_targets": []
  },
  "inviter": {
    "email": "dave.dunn@example.com",
    "title": "",
    "first_name": "Dave",
    "last_name": "Dunn",
    "language": "en"
  }
```

<!-- end:fromfile:mail-worker-protocol/unregistered_event_cancellation.json.md -->

### External user notifications

#### Invite

<!-- begin:fromfile:mail-worker-protocol/external_event_invite.json.md -->

```json
{
  "version": "1",
  "message": "external_event_invite",
  "invitee": {
    "email": "charlie.cooper@example.com"
  },
  "event": {
    "id": "00000000-0000-0000-0000-0000abadcafe",
    "name": "Weekly teammeeting",
    "created_at": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "start_time": null,
    "end_time": null,
    "rrule": null,
    "description": "The team's regular weekly meeting",
    "room": {
      "id": "00000000-0000-0000-0000-0000abcdef99",
      "password": "v3rys3cr3t"
    },
    "call_in": {
      "sip_tel": "+99-1234567890",
      "sip_id": "1234567890",
      "sip_password": "9876543210"
    },
    "revision": 3,
    "shared_folder": {
      "read": {
        "url": "https://cloud.example.com/shares/abc123",
        "password": "v3rys3cr3t"
      }
    },
    "adhoc_retention_seconds": null,
    "streaming_targets": []
  },
  "inviter": {
    "email": "dave.dunn@example.com",
    "title": "",
    "first_name": "Dave",
    "last_name": "Dunn",
    "language": "en"
  },
  "invite_code": "5b5f9cf0-86a8-4e5a-bfac-b05c34c8a20b"
```

<!-- end:fromfile:mail-worker-protocol/external_event_invite.json.md -->

#### Update

<!-- begin:fromfile:mail-worker-protocol/external_event_update.json.md -->

```json
{
  "version": "1",
  "message": "external_event_update",
  "invitee": {
    "email": "charlie.cooper@example.com"
  },
  "event": {
    "id": "00000000-0000-0000-0000-0000abadcafe",
    "name": "Weekly teammeeting",
    "created_at": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "start_time": null,
    "end_time": null,
    "rrule": null,
    "description": "The team's regular weekly meeting",
    "room": {
      "id": "00000000-0000-0000-0000-0000abcdef99",
      "password": "v3rys3cr3t"
    },
    "call_in": {
      "sip_tel": "+99-1234567890",
      "sip_id": "1234567890",
      "sip_password": "9876543210"
    },
    "revision": 3,
    "shared_folder": {
      "read": {
        "url": "https://cloud.example.com/shares/abc123",
        "password": "v3rys3cr3t"
      }
    },
    "adhoc_retention_seconds": null,
    "streaming_targets": []
  },
  "event_exception": {
    "exception_date": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "kind": "modified",
    "title": "Another weekly meeting",
    "description": null,
    "is_all_day": null,
    "starts_at": null,
    "ends_at": null
  },
  "inviter": {
    "email": "dave.dunn@example.com",
    "title": "",
    "first_name": "Dave",
    "last_name": "Dunn",
    "language": "en"
  },
  "invite_code": "5b5f9cf0-86a8-4e5a-bfac-b05c34c8a20b"
```

<!-- end:fromfile:mail-worker-protocol/external_event_update.json.md -->

#### Uninvite

<!-- begin:fromfile:mail-worker-protocol/external_event_uninvite.json.md -->

```json
{
  "version": "1",
  "message": "external_event_uninvite",
  "invitee": {
    "email": "charlie.cooper@example.com"
  },
  "event": {
    "id": "00000000-0000-0000-0000-0000abadcafe",
    "name": "Weekly teammeeting",
    "created_at": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "start_time": null,
    "end_time": null,
    "rrule": null,
    "description": "The team's regular weekly meeting",
    "room": {
      "id": "00000000-0000-0000-0000-0000abcdef99",
      "password": "v3rys3cr3t"
    },
    "call_in": {
      "sip_tel": "+99-1234567890",
      "sip_id": "1234567890",
      "sip_password": "9876543210"
    },
    "revision": 3,
    "shared_folder": {
      "read": {
        "url": "https://cloud.example.com/shares/abc123",
        "password": "v3rys3cr3t"
      }
    },
    "adhoc_retention_seconds": null,
    "streaming_targets": []
  },
  "inviter": {
    "email": "dave.dunn@example.com",
    "title": "",
    "first_name": "Dave",
    "last_name": "Dunn",
    "language": "en"
  }
```

<!-- end:fromfile:mail-worker-protocol/external_event_uninvite.json.md -->

#### Cancellation

<!-- begin:fromfile:mail-worker-protocol/external_event_cancellation.json.md -->

```json
{
  "version": "1",
  "message": "external_event_cancellation",
  "invitee": {
    "email": "charlie.cooper@example.com"
  },
  "event": {
    "id": "00000000-0000-0000-0000-0000abadcafe",
    "name": "Weekly teammeeting",
    "created_at": {
      "time": "2024-07-05T17:02:42Z",
      "timezone": "Europe/Berlin"
    },
    "start_time": null,
    "end_time": null,
    "rrule": null,
    "description": "The team's regular weekly meeting",
    "room": {
      "id": "00000000-0000-0000-0000-0000abcdef99",
      "password": "v3rys3cr3t"
    },
    "call_in": {
      "sip_tel": "+99-1234567890",
      "sip_id": "1234567890",
      "sip_password": "9876543210"
    },
    "revision": 3,
    "shared_folder": {
      "read": {
        "url": "https://cloud.example.com/shares/abc123",
        "password": "v3rys3cr3t"
      }
    },
    "adhoc_retention_seconds": null,
    "streaming_targets": []
  },
  "inviter": {
    "email": "dave.dunn@example.com",
    "title": "",
    "first_name": "Dave",
    "last_name": "Dunn",
    "language": "en"
  }
```

<!-- end:fromfile:mail-worker-protocol/external_event_cancellation.json.md -->
