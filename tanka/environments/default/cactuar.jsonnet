local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';

{
  // use locals to extract the parts we need
  local deploy = k.apps.v1.deployment,
  local container = k.core.v1.container,
  local port = k.core.v1.containerPort,
  local service = k.core.v1.service,

  local clusterRole = k.rbac.v1.clusterRole,
  local policyRule = k.rbac.v1.policyRule,
  local apiGroup = k.meta.v1.apiGroup,
  // defining the objects:
  cactuar: {
    // deployment constructor: name, replicas, containers
    deployment: deploy.new(
                  name=$._config.cactuar.name,
                  replicas=1,
                  podLabels={
                    'app.kubernetes.io/name': $._config.cactuar.name,
                    'app.kubernetes.io/instance': $._config.cactuar.name,
                  },
                  containers=[
                    // container constructor
                    container.new(
                      $._config.cactuar.name,
                      $._config.cactuar.image
                    ) +
                    container.withPorts(  // add ports to the container
                      [port.new($._config.cactuar.ports.http.name, $._config.cactuar.ports.http.port)]  // port constructor
                    ) +
                    container.readinessProbe.httpGet.withPath('/ready') + container.readinessProbe.httpGet.withPort('http'),
                  ],
                ) +
                deploy.spec.template.metadata.withAnnotations($._config.cactuar.annotations),
    clusterRole: clusterRole.new(name=$._config.cactuar.name) +
                 clusterRole.withRulesMixin(
                   policyRule.withApiGroups('cactuar.rs') +
                   policyRule.withResources('*') +
                   policyRule.withVerbs('*')
                 ) +
                 clusterRole.withRulesMixin(
                   policyRule.withApiGroups('events.k8s.io') +
                   policyRule.withResources('events') +
                   policyRule.withVerbs(['create', 'get', 'list', 'watch'])
                 ) +
                 clusterRole.withRulesMixin(
                   policyRule.withApiGroups('' /* the core api group is the empty string: '' */) +
                   policyRule.withResources('configmaps') +
                   policyRule.withVerbs(['create', 'get', 'list', 'watch', 'update', 'patch', 'delete'])
                 ) +
                 clusterRole.withRulesMixin(
                   policyRule.withApiGroups('monitoring.coreos.com') +
                   policyRule.withResources('prometheusrules') +
                   policyRule.withVerbs(['create', 'get', 'list', 'watch', 'update', 'patch', 'delete'])
                 ),
  },
}
