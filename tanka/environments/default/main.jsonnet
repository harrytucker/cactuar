(import 'cactuar.jsonnet') +
{
  _config:: {
    cactuar: {
      name: 'cactuar',
      image: 'cactuar:latest',
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
    },
  },
}
