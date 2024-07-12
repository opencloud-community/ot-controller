# OpenTalk Controller API Specification

This folder contains the OpenTalk API specification formalized in OpenAPI
format.

## Checking the consistency with Spectral

[Stoplight Spectral](https://stoplight.io/open-source/spectral) is a linter tool
for structured data such as JSON and YAML. It contains built-in support to
ensure the consistency of an OpenAPI specfication. These checks go far beyond
what most other linters detect, resulting in significantly higher consistency of
the OpenAPI specification.

### Running the checks locally

#### Prerequisites

The subsequent commands assume that the project root is stored in the environment variable `PROJECT_ROOT` like this:

```bash
export PROJECT_ROOT="/path/to/opentalk/controller"
```

Alternatively if the project root is the current directory:

```bash
export PROJECT_ROOT="$(pwd)"
```

#### With `spectral` installed

```bash
spectral lint --ruleset "$PROJECT_ROOT"/ci/spectral/openapi.yml "$PROJECT_ROOT"/api/controller/frontend_api.yaml
```

#### With the `stoplight/spectral` Docker image

```bash
docker run --rm -it -v "$PROJECT_ROOT":/tmp stoplight/spectral lint --ruleset /tmp/ci/spectral/openapi.yml /tmp/api/controller/frontend_api.yaml
```
