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

So how does the upstream server know when to scale up? Any scaler that works with prometheus metrics will do, like KEDA core, or you may prefer a vanilla HPA.

The reported metrics are as follows:

- *agimo_requests_total { host = "example.com" }*: (COUNTER)the total number of requests for a host
- *agimo_active_requests { host = "example.com" }*: (GAUGE)the number of active requests(waiting for the upstream) for a host

# Installation

## Helm(Recommended)

```bash
helm repo add agimo https://agimo.onichandame.com
helm install agimo agimo/agimo
```

<details>
<summary>Expand example values</summary>

```yaml
config: |
  timeout = '30s'
  [[services]]
  host = 'example.com'
  type = 'deployment'
  namespace = 'example'
  name = 'example'
  service_name = 'example'
  service_port = 80
podAnnotations:
  prometheus.io/scrape: "true"
  prometheus.io/scheme: "http"
  prometheus.io/path: "/"
  prometheus.io/port: "9090"
prometheus:
  address: http://prometheus-server.prometheus.svc.cluster.local
```

</details>

The prometheus annotations are required in order for prometheus scraper to find the agimo pods. **It is recommended to set the scrape interval shorter than default(1m), like 5s or so.**

## Manual(local)

```bash
# Example command
agimo --prometheus-address http://prometheus --conf ./config.toml
```

An example config will be:

```toml
# the default timeout waiting for the upstream to scale up from zero
timeout = '5s'

[[services]]
host = "example.com"
type = "deploy" # "deployment" | "statefulset" | "deploy" | "sts"
namespace = "default"
name = "example"
service_name = "example"
service_port = 80
```

# Usage

## KEDA

<details>
<summary>Expand example ScaledObject</summary>

```yaml
apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata:
  name: example
  namespace: example
spec:
  cooldownPeriod: 30
  maxReplicaCount: 1 # scale up when there are requests
  minReplicaCount: 0 # scale to zero when there are no requests
  pollingInterval: 2
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: example
  triggers:
  - metadata:
      # use the rate of requests in the last 1m and the number of active requests(waiting for the upstream/Websocket/SSE) as the metric for scaling.
      query: sum by (host) ((rate(agimo_requests_total{host="example.com"}[1m])) + (agimo_active_requests{host="example.com"}))
      serverAddress: http://prometheus-server.prometheus.svc.cluster.local
      threshold: "1000"
    type: prometheus
```

</details>