# FHIR

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

## Notes

- First insert is relatively slow, because the JSON schema must be compiled
  first
  - first entity takes around 600ms
  - after that `fhir_put` takes around 3-4ms
  - could be improved by compiling the schema when starting
- fastrace global exporter thread is not stopped when extension is dropped
