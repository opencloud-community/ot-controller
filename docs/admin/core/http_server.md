# HTTP server

The OpenTalk Controller provides its service to clients through a built-in HTTP
server.

Services provided:

- [`v1` REST API](https://opentalk.eu/docs/developer/controller/rest/) under `/v1`
- [Signaling](https://opentalk.eu/docs/developer/controller/signaling/) for meetings under `/signaling`
- [Metrics](logging/metrics.md) under `/metrics`

## Configuration

The section in the [configuration file](configuration.md) is called `http`.

| Field  | Type                                    | Required | Default value | Description                                                                                    |
| ------ | --------------------------------------- | -------- | ------------- | ---------------------------------------------------------------------------------------------- |
| `addr` | `string`                                | no       | -             | IP address or hostname to which to listen for incoming connections                             |
| `port` | `uint`                                  | no       | `11311`       | TCP port number where the HTTP server can be reached                                           |
| `tls`  | [TLS configuration](#tls-configuration) | no       | -             | When present, the HTTP server will use TLS, when absent it will serve under a plain connection |

### Listening address

By default, the service will accept requests on both the IPv4 and IPv6
interfaces if either a hostname is set for `addr`, or if no `addr` value is set
at all.

The exception to this rule is `"::0"` which will bind to both the IPv4
`UNSPECIFIED` address and the IPv6 `UNSPECIFIED` address at the same time,
accepting requests on any address for both protocols. If the operating system
provides no IPv6 support, or the service should not bind to an IPv6 interface,
`"0.0.0.0"` can be used instead, which will only bind to the IPv4 `UNSPECIFIED`
address.

A hostname or fully qualified domain name will bind to whatever the name
resolution returns, either one or both IP protocols.

An explicit IPv4 or IPv6 address, will bind exactly to the corresponding IP protocol.

### TLS configuration

| Field         | Type     | Required | Default value | Description                                                                 |
| ------------- | -------- | -------- | ------------- | --------------------------------------------------------------------------- |
| `certificate` | `string` | yes      | -             | Path to the file containing the TLS certificate in DER-encoded x.509 format |
| `private_key` | `string` | yes      | -             | Path to the file containing the private TLS key in pkcs8 format             |

### Examples

#### Plain HTTP on all addresses

```toml
[http]
port = 80
```

#### Plain HTTP on localhost only (IPv4 and IPv6 if available)

```toml
[http]
addr = "localhost"
port = 80
```

#### Plain HTTP on an IPv4 address

```toml
[http]
addr = "192.0.2.0"
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

#### HTTP over TLS on an IPv6 address

```toml
[http]
addr = "2001:0DB8::1337:DEAD:CAFE"
port = 443

[http.tls]
certificate = "/etc/ssl/certs/example.org.pem"
private_key = "/etc/ssl/keys/example.org.key"
```
