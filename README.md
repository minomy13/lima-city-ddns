# lima-city-ddns
This little node.js app allows you to have Dynamic DNS for your [lima-city](https://www.lima-city.de) DNS server.
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
