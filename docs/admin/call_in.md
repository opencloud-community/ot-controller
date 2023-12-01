---
sidebar_position: 110
---

# Call-in

The call-in module provides a user-friendly solution that not only allows
participants to connect via telephone but also displays the necessary phone
number, ensuring a straightforward and accessible joining process.

## Configuration

The section in the [configuration file](configuration.md) is called `call_in`.

| Field                  | Type     | Required | Default value | Description                                            |
| ---------------------- | -------- | -------- | ------------- | ------------------------------------------------------ |
| `tel`                  | `string` | yes      | -             | The Phone number which will be displayed to the user   |
| `enable_phone_mapping` | `bool`   | yes      | -             | Enable the mapping of user names to their phone number |
| `default_country_code` | `string` | yes      | -             | The default country code as Alpha-2 code (ISO 3166)    |

### Examples

#### Example with phone mapping

Enable the mapping of user names to their phone number. This requires the OIDC
provider to have a phone number field configured for their users.

```toml
[call_in]
tel="03012345678"
enable_phone_mapping=true
default_country_code="DE"
```

#### Example without phone mapping

```toml
[call_in]
tel="03012345678"
enable_phone_mapping=false
default_country_code="DE"
```
