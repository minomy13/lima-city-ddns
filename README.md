# lima-city-ddns
This little node.js app allows you to have Dynamic DNS for your [lima-city](https://www.lima-city.de) DNS server.
Every minute (and on startup) the app will check, if you have a new public IP address. It is using the [ipfy](https://www.ipify.org) API for that task. If so, it will update you DNS record.

## Gaining required values
At first, you should create an API-Key in the lima-city settings. You should assign the `domains.admin` and `dns.admin` permissions. Then you can execute `curl https://www.lima-city.de/usercp/domains.json -u api:<YOUR_API_KEY>` and read the domain ID from the JSON. After that you can get the ID of the A record you want to update by reading the response of `curl https://www.lima-city.de/usercp/domains/<YOUR_DOMAIN_ID>/records.json -u api:<YOUR_API_KEY>`.

### Formatting your DOMAIN_DATA
To provide your domain IDs and record IDs to the DDNS server, you need to form a special string. The syntax is as following: `<YOUR_DOMAIN_ID_1>:<YOUR_RECORD_ID_1>,<YOUR_RECORD_ID_n>;<YOUR_DOMAIN_ID_n>:<...>` There MUST NOT be a semicolon at the end!

By that, you can update multiple records of multiple domains at the same time.

## Docker Compose
For example, you could design your Docker compose like that:

```yml
services:
  ddns:
    image: "ghcr.io/minomy13/lima-city-ddns:main"
    restart: always
    environment:
      AUTH: "<YOUR_API_KEY>"
      DOMAIN_DATA: "<YOUR_DOMAIN_DATA>"
```
