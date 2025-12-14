# FHIR

Documentation of the API endpoints is available via Scalar UI at `<server>/docs`
path. There is one instance deployed at `https://rome.neon-opah.ts.net/docs`,
which you can use for testing, and exploring the api.

The server theoretically supports any FHIR resource, but only the `Patient`
resource is indexed, so it can be searched.

`Patient` supports the following search parameters:

- `birth_date`
- `name`
- `gender`

## Tracing

The extension contains basic tracing support that can be used to measure and
monitor the performance of each function cal.

To enable tracing, run `docker compose up -d` and append the following
parameters to your `postgresql.conf` file:

```text
fhir.jaeger_enabled = 'true'
fhir.jaeger_host = '127.0.0.1:6831'
```

Then reload the extension and start tracing!

## FHIR standard

The HTTP part is only loosely following the official FHIR specification, because
the standard is very complex in my opinion, and not worth it in a simple project
like this.

Most notably responses from the endpoints, like the search route, will not
return the FHIR standard responses. Instead they return a simple to use json
structure.

## Notes

- First insert is relatively slow, because the JSON schema must be compiled
  first
  - first entity takes around 600ms
  - after that `fhir_put` takes around 3-4ms
  - could be improved by compiling the schema when starting
- fastrace global exporter thread is not stopped when extension is dropped
- The search endpoint only supports one single search paramater right now
- Only eq, ne, gt, ge, lt and le FHIR operators are supported
- Implement custom postgres error codes, so the API can handle certain errors
  (like unknown search key) properly
