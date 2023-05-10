local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';

{
  // use locals to extract the parts we need
  local deploy = k.apps.v1.deployment,
  local container = k.core.v1.container,
  local port = k.core.v1.containerPort,
  local service = k.core.v1.service,
  local servicePort = k.core.v1.servicePort,
  local serviceAccount = k.core.v1.serviceAccount,

  local clusterRole = k.rbac.v1.clusterRole,
  local clusterRoleBinding = k.rbac.v1.clusterRoleBinding,
  local policyRule = k.rbac.v1.policyRule,
  local subject = k.rbac.v1.subject,

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
                deploy.spec.template.metadata.withAnnotations($._config.cactuar.annotations) +
                deploy.spec.template.spec.withServiceAccountName($._config.cactuar.name),
    service: service.new(
      name=$._config.cactuar.name,
      selector=$._config.cactuar.labels,
      ports=servicePort.newNamed(
        name=$._config.cactuar.ports.http.name,
        port=$._config.cactuar.ports.http.port,
        targetPort=$._config.cactuar.ports.http.name
      ) + servicePort.withProtocol('TCP'),
    ),
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
                   policyRule.withApiGroups(''/* the core api group is the empty string: '' */) +
                   policyRule.withResources('configmaps') +
                   policyRule.withVerbs(['create', 'get', 'list', 'watch', 'update', 'patch', 'delete'])
                 ) +
                 clusterRole.withRulesMixin(
                   policyRule.withApiGroups('monitoring.coreos.com') +
                   policyRule.withResources('prometheusrules') +
                   policyRule.withVerbs(['create', 'get', 'list', 'watch', 'update', 'patch', 'delete'])
                 ),
    clusterRoleBinding: clusterRoleBinding.new(name=$._config.cactuar.name) +
                        clusterRoleBinding.withSubjects(
                          subject.fromServiceAccount(self.serviceAccount) +
                          subject.withNamespace($._config.cactuar.namespace)
                        ) +
                        clusterRoleBinding.bindRole(self.clusterRole),
    serviceAccount: serviceAccount.new($._config.cactuar.name) +
                    serviceAccount.metadata.withNamespace($._config.cactuar.namespace),
    local crdSpec = importstr 'crdspec.yaml',
    crd: std.native('parseYaml')(crdSpec),
  },
}
