---
sidebar_position: 102
title: HTTP Server
---

# OpenTalk HTTP server

The OpenTalk Controller provides its service to clients through a built-in HTTP
server.

Services provided:

- [`v1` REST API](https://opentalk.eu/docs/developer/controller/rest/) under `/v1`
- [Signaling](https://opentalk.eu/docs/developer/controller/signaling/) for meetings under `/signaling`
- [Metrics](metrics.md) under `/metrics`

## Configuration

The section in the [configuration file](configuration.md) is called `http`.

| Field  | Type                                    | Required | Default value | Description                                                                                    |
| ------ | --------------------------------------- | -------- | ------------- | ---------------------------------------------------------------------------------------------- |
| `port` | `uint`                                  | no       | `11311`       | TCP port number where the HTTP server can be reached                                           |
| `tls`  | [TLS configuration](#tls-configuration) | no       | -             | When present, the HTTP server will use TLS, when absent it will serve under a plain connection |

### TLS configuration

| Field         | Type     | Required | Default value | Description                                                                 |
| ------------- | -------- | -------- | ------------- | --------------------------------------------------------------------------- |
| `certificate` | `string` | yes      | -             | Path to the file containing the TLS certificate in DER-encoded x.509 format |
| `private_key` | `string` | yes      | -             | Path to the file containing the private TLS key in pkcs8 format             |

### Examples

#### Plain HTTP

```toml
[http]
port = 80
```

#### HTTP over TLS

```toml
[http]
port = 443

[http.tls]
certificate = "/etc/ssl/certs/example.org.pem"
private_key = "/etc/ssl/keys/example.org.key"
```
