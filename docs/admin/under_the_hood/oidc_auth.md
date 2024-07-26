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
    I-->>F: Returns ID Token and Access Token

    F->>C: Logs into controller using ID Token<br>POST /auth/login
    C->>C: Validate ID Token:<br>Check Signature against Provider PubKey<br>Cannot be expired
    Note over C: The endpoint does not authenticate the user<br>but stores the ID Token inside a DB Table to check expiration later
    C-->>F: Returns
    F->>U: Login Complete!

    opt Normal REST calls
    U->>F: Creates new Meeting
    F->>C: POST /v1/event

    C->>C: Parse JWT AccessToken<br>Check Signature and expiration
    C->>C: Verify permissions<br>by looking at groups inside AccessToken
    C->>I: Verify token against<br>token_instrospect endpoint
    I-->>C: Returns token info
    C->>C: Process the info and check if token is still active.
    C->>C: Process request

    C-->F: Returns newly created Meeting
    F->>U: Shows Meeting
    end
```
