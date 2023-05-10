(import 'cactuar.jsonnet') +
{
  _config:: {
    cactuar: {
      name: 'cactuar',
      image: 'registry.localhost:12345/cactuar:tilt-a86976fecee1ad34',
      ports: {
        http: {
          port: 8080,
          name: 'http',
        },
      },
      annotations: {
        'prometheus.io/path': '/metrics',
        'prometheus.io/port': '8080',
        'prometheus.io/scrape': 'true',
      },
      labels: {
        'app.kubernetes.io/name': 'cactuar',
        'app.kubernetes.io/instance': 'cactuar',
      },
      namespace: "default",
    },
  },
}
