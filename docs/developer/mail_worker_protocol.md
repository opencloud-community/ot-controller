# Mail-Worker Protocol

This sections describes the schema and lists a few examples for messages to the
[OpenTalk SMTP Mailer](https://docs.opentalk.eu/admin/smtp-mailer).

## Schema

<!-- begin:fromfile:mail-worker-protocol/schema.json.md -->

```json
{
  "components": {
    "schemas": {
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
            "allOf": [
              {
                "$ref": "#/components/schemas/EventException"
              }
            ],
            "nullable": true
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
        "description": "Versioned Mail Task Protocol",
        "discriminator": {
          "propertyName": "version"
        }
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
            "allOf": [
              {
                "$ref": "#/components/schemas/EventException"
              }
            ],
            "nullable": true
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
            "allOf": [
              {
                "$ref": "#/components/schemas/EventException"
              }
            ],
            "nullable": true
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
        "description": "The different kinds of MailTasks that are currently supported",
        "discriminator": {
          "propertyName": "message"
        }
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
