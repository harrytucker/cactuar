local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';

{
  local newPolicyRule(apiGroups, resources, verbs) = k.rbac.v1.policyRule.new() +
                                                     k.rbac.v1.policyRule.withApiGroups(apiGroups) +
                                                     k.rbac.v1.policyRule.withResources(resources) +
                                                     k.rbac.v1.policyRule.withVerbs(verbs),

  cactuar: newPolicyRule('cactuar.rs', '*', '*'),

  events: newPolicyRule('events.k8s.io', 'events', ['create', 'get', 'list', 'watch']),

  configMaps: newPolicyRule('', 'configmaps', ['create', 'get', 'list', 'watch', 'update', 'patch', 'delete']),

  prometheusRules: newPolicyRule('monitoring.coreos.com', 'prometheusrules', ['create', 'get', 'list', 'watch', 'update', 'patch', 'delete']),
}
