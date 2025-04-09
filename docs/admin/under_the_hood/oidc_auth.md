# OIDC Authentication Flow for OpenTalk Controller WebAPI endpoints

This diagram describes the flow of an authentication against the OIDC provider
for accessing the
[OpenTalk Controller WebAPI](https://docs.opentalk.eu/developer/controller/rest/).

```mermaid
sequenceDiagram
    participant U as User
    participant F as Frontend
    participant C as Controller API
    participant I as OIDC Provider

    U->>F: Clicks login
    F->>I: Performs OIDC Login
    I-->>F: Returns Access Token

    opt Normal REST calls
    U->>F: Creates new Meeting
    F->>C: POST /v1/event with HTTP header:<br>Authentication: Bearer <AccessToken>

    opt cached per AccessToken
        Note over C: The OIDC provider must either implement the introspect endpoint<br>or have AccessTokens in the JWT format<br>If neither is the case an error is always returned to the user
        alt OIDC Provider supports introspect
            C->>I: Verify token against<br>introspect endpoint
            I-->>C: Returns token info
        else AccessToken is a JWT
            C->>C: Verify AccessToken<br>Check Signature and expiration
        end


        C->>I: Query userinfo endpoint using the AccessToken
        I-->>C: Returns user information

        C->>C: Query database for the user,<br>creates one if it doesn't exist.

    end

    C->>C: Process request

    C-->F: Returns newly created Meeting
    F->>U: Shows Meeting
    end
```
