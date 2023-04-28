# Build the cactuar Dockerfile target to bring code changes up-to-date for the
# Helm chart.
#
# Keep in mind, while cargo-chef caches dependency builds, if you change a
# dependency version or install a new dependency, you may trigger a full
# rebuild, which will take significantly longer than a cached build.
docker_build(
    "cactuar",
    context=".",
    dockerfile="./Dockerfile",
    ignore=["./helm/"]
)

update_settings(k8s_upsert_timeout_secs = 180)

load('ext://helm_resource', 'helm_resource', 'helm_repo')
helm_repo('prometheus-community', 'https://prometheus-community.github.io/helm-charts')
helm_resource('kube-prometheus-stack', 'prometheus-community/kube-prometheus-stack')

k8s_resource(
    workload='kube-prometheus-stack',
    port_forwards=[
        # port_forward takes the form: (local_port, container_port)
        port_forward(9090, 9090, name='Prometheus UI'),
        port_forward(9093, 9093, name='Alert Manager UI'),
        port_forward(3000, 3000, name='Grafana UI'),
    ],
)

# Deploy the local Helm chart to your running Kubernetes cluster. Note that this
# will depend on you using some kind of local cluster setup tool, such as K3D,
# Kind, Minikube, or other.
#
# Note: Cactuar relies on being able to create a ClusterRole and
# ClusterRoleBinding in order to modify CRD specs and control its own resources,
# so if your cluster restricts these permissions you will likely run into
# issues.
k8s_yaml(helm("./helm/cactuar"))

# Expose tokio-console port
k8s_resource(
    "chart-cactuar",
    port_forwards=[
        # port_forward takes the form: (local_port, container_port)
        port_forward(6669, 6669, name='Tokio Console (Port 6669)')
    ],
)
