# Agimo

Yet another scale-to-zero solution for k8s.

## Why?

We already have Knative serving and KEDA HTTP add on, why another project doing the same? Here are my requirements:

- Websocket support([~~KEDA HTTP add on~~](https://github.com/kedacore/http-add-on/issues/654))
- Work with k8s native deployment/statefulset(~~Knative serving~~)

## Architecture

So here is the plan:

1. All incoming requests should be proxied by agimo
2. Based on the service configuration, agimo finds the number of ready replicas of the upstream service before passing through the request
3. if the upstream is ready, go to 5, otherwise go to 4
4. wait for the upstream server to be up running
5. pass through the request to the upstream server

So how does the upstream server know when to scale up? KEDA come into play. Don't make it wrong, it's only the KEDA core that is needed, not the immature KEDA HTTP add on.

When the requests comes in, agimo reports the metrics to the prometheus store. KEDA has a battle-tested prometheus scaler which can be used to scale the upstream service based on custom prometheus metrics.

The reported metrics are as follows:

- *requests_total { host = "example.com" }*: the total number of requests for a host
- *pending_requests_total { host = "example.com" }*: the number of pending requests(waiting for the upstream) for a host
- *active_requests_total { host = "example.com" }*: the number of active requests(passed to the upstream) for a host

An example scaler definition will be:

```yaml
triggers:
- type: prometheus
  metadata:
    # Required fields:
    serverAddress: http://<prometheus-host>:9090
    query: max_over_time(requests_total{host="example.com"}[30s]) # Note: query must return a vector/scalar single element response
    threshold: '100'
    # Optional fields:
    namespace: example-namespace  # for namespaced queries, eg. Thanos
    ignoreNullValues: false # Default is `true`, which means ignoring the empty value list from Prometheus. Set to `false` the scaler will return error when Prometheus target is lost
```