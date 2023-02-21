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
k8s_resource("chart-cactuar", port_forwards="6669:6669")
