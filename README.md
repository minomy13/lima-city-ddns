# lima-city-ddns
This little node.js app allows you to have Dynamic DNS for your [lima-city](https://www.lima-city.de) DNS server.
Every minute (and on startup) the app will check, if you have a new public IP addess. It is using the [ipfy](https://www.ipify.org) API for that task. If so, it will update you DNS record.
## Gaining required values

## Docker Compose
```yml
services:
  ddns:
    image: "ghcr.io/minomy13/ddns:main"
    restart: always
    environment:
      DOMAIN_ID: "YOUR_DOMAIN_ID"
      PROVIDER: "lima_city"
      AUTH: "YOUR_AUTH_TOKEN"
      RECORD_ID: YOUR_RECORD_ID
```
