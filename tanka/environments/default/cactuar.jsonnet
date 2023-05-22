local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';

{
  local name = $._config.cactuar.name,
  local image = $._config.cactuar.image,
  local portName = $._config.cactuar.ports.http.name,
  local portNumber = $._config.cactuar.ports.http.port,
  local namespace = $._config.cactuar.namespace,
  local labels = $._config.cactuar.labels,
  local annotations = $._config.cactuar.annotations,

  local newContainer(name, image) = k.core.v1.container.new(name, image) +
                                    k.core.v1.container.withPorts([k.core.v1.containerPort.new(portName, portNumber)]) +
                                    k.core.v1.container.readinessProbe.httpGet.withPath('/ready') +
                                    k.core.v1.container.readinessProbe.httpGet.withPort('http'),

  local newPolicyRule(apiGroups, resources, verbs) = k.rbac.v1.policyRule.new() +
                                                     k.rbac.v1.policyRule.withApiGroups(apiGroups) +
                                                     k.rbac.v1.policyRule.withResources(resources) +
                                                     k.rbac.v1.policyRule.withVerbs(verbs),


  cactuar: {
    deployment: k.apps.v1.deployment.new(name=name, replicas=1, podLabels=labels, containers=[newContainer(name, image)]) +
                k.apps.v1.deployment.spec.template.metadata.withAnnotations(annotations) +
                k.apps.v1.deployment.spec.template.spec.withServiceAccountName(name),

    service: k.core.v1.service.new(name, labels, [k.core.v1.servicePort.newNamed(portName, portNumber, portName) + k.core.v1.servicePort.withProtocol('TCP')]),

    clusterRole: k.rbac.v1.clusterRole.new(name) +
                 k.rbac.v1.clusterRole.withRulesMixin(newPolicyRule('cactuar.rs', '*', '*')) +
                 k.rbac.v1.clusterRole.withRulesMixin(newPolicyRule('events.k8s.io', 'events', ['create', 'get', 'list', 'watch'])) +
                 k.rbac.v1.clusterRole.withRulesMixin(newPolicyRule('', 'configmaps', ['create', 'get', 'list', 'watch', 'update', 'patch', 'delete'])) +
                 k.rbac.v1.clusterRole.withRulesMixin(newPolicyRule('monitoring.coreos.com', 'prometheusrules', ['create', 'get', 'list', 'watch', 'update', 'patch', 'delete'])),

    clusterRoleBinding: k.rbac.v1.clusterRoleBinding.new(name) +
                        k.rbac.v1.clusterRoleBinding.withSubjects(k.rbac.v1.subject.fromServiceAccount(k.core.v1.serviceAccount.new(name)) + k.rbac.v1.subject.withNamespace(namespace)) +
                        k.rbac.v1.clusterRoleBinding.bindRole(k.rbac.v1.clusterRole.new(name)),

    serviceAccount: k.core.v1.serviceAccount.new(name) + k.core.v1.serviceAccount.metadata.withNamespace(namespace),
    crd: std.native('parseYaml')(importstr 'crdspec.yaml'),
  },
}
