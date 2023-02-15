# Cactuar

[![Rust](https://github.com/harrytucker/cactuar/actions/workflows/rust.yml/badge.svg)](https://github.com/harrytucker/cactuar/actions/workflows/rust.yml)

![Wanted!!](https://static.wikia.nocookie.net/finalfantasy/images/6/6e/Cactuar_FFVIII_Color_Art.jpg/revision/latest?cb=20130315023650)

_**"Becoming a fine cactus, protecting the desert- those who forsake this duty flee there."**_

## What is this?

Often, services deployed within a service mesh may want to deploy Prometheus
alerts that are similar to those of other services. Implementing these same
alerts over and over again, often using private metric implementations take time
to understand, build, and deploy.

What if all these services, deployed in the same mesh had a bunch of common
metrics from which we could derive alerts that would work across them all?

_One can hope..._

## Local Development

Cactuar is intended to run within a Kubernetes cluster, with a Helm chart
provided for this purpose. You can deploy the Helm chart yourself, but Cactuar
also supports the use of [Tilt](https://tilt.dev) for running the whole
application, with local auto-reloading.

### Requirements

Ensure you have a local Kubernetes cluster running that you can use. A
[K3D](https://k3d.io) config file is provided as a known working configuration,
which you can deploy using the following:

```sh
k3d cluster create --config k3d-conf.yaml
```

So long as the Kubernetes cluster allows the usage of Custom Resource
Definitions, Cactuar should function. Other possible ways to run a local
cluster:

- Kind
- Minikube

### Tilt

To use Tilt, simply open a shell in the Cactuar root directory, and then run:

```sh
tilt up
```

You can take down the Tilt instance, conversely, with:

```sh
tilt down
```

Tilt will take care of building the Dockerfile, templating the Helm chart, and
deploying it to a local Kubernetes cluster.

### Manual

A Cargo makefile is provided for building the Docker image yourself if you need
it. To build the Docker image, run the following task:

```sh
# make sure you have cargo-make installed
cargo install --force cargo-make

# run the makefile task
cargo make docker
```

## What's with the name?

Cactuars are enemies from the Final Fantasy series of games. They're twitchy,
and often run away when provoked, so I guess you could say they are quite
_alert (ba dum tss)_.
