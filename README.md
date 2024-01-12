# lima-city-ddns
This app allows you to have Dynamic DNS for your [lima-city](https://www.lima-city.de) DNS server.

## Gaining required values
At first, you should create an API-Key in the lima-city settings. You should assign the `domains.admin` and `dns.admin`
permissions. Then you can execute `curl https://www.lima-city.de/usercp/domains.json -u api:<YOUR_API_KEY>` and read the
domain ID from the JSON. After that you can get the ID of the A record you want to update by reading the response of
`curl https://www.lima-city.de/usercp/domains/<YOUR_DOMAIN_ID>/records.json -u api:<YOUR_API_KEY>`.

### Formatting your DOMAIN_DATA
To provide your domain IDs and record IDs to the DDNS server, you need to form a special string. The syntax is as
following: `<YOUR_DOMAIN_ID_1>:<YOUR_RECORD_ID_1>,<YOUR_RECORD_ID_n>;<YOUR_DOMAIN_ID_n>:<...>` There MUST NOT be
a semicolon at the end!

By that, you can update multiple records of multiple domains at the same time.

## Docker Compose
### External API mode
This mode fetches your public IP address from the [ipfy](https://www.ipify.org) API every minute. If there should be a new one, it will
update your records.
For example, you could design your Docker compose like that:

```yml
services:
  ddns:
    image: "ghcr.io/minomy13/lima-city-ddns:latest"
    restart: always
    environment:
      AUTH: "<YOUR_API_KEY>"
      DOMAIN_DATA: "<YOUR_DOMAIN_DATA>"
```

### Router mode
This mode hosts a web server waiting for a DDNS request of your router. I can provide you with precise setup guides for
the [AVM FritzBox]() and the [UniFi UDM Pro](). [COMING SOON]
For any other router you should be able to figure it out yourself with the following information:
- The Update-URL should be as following: `<address>/?password=<password>&ip=<ip_address>`. The `password` and
  `ip_address` fields vary depending on your router. For the FritzBox for example they are `<pass>` and `<ipaddr>`, for
  [inadyn](https://github.com/troglobit/inadyn)-based software however, like on the UDM Pro, it would be `%p` and `%i`.
- The password is the one you'll set the `PASSWORD` environment variable to. Must be URL encoded -
  [this website](https://www.urlencoder.org) worked perfectly for me.
- The username really doesn't matter - get creative! 😉
- The same applies to the host.

So, your docker compose could look like that:
```yml
services:
  ddns:
    image: "ghcr.io/minomy13/lima-city-ddns:latest"
    restart: always
    environment:
      AUTH: "<YOUR_API_KEY>"
      DOMAIN_DATA: "<YOUR_DOMAIN_DATA>"
      MODE: "router"
      PASSWORD: "<YOUR_CUSTOM_PASSWORD>"
```

## Environment variables
| Variable      | Usage                                                                                                                                                    | Default        | Required            |
|---------------|----------------------------------------------------------------------------------------------------------------------------------------------------------|----------------|---------------------|
| `AUTH`        | lima-city auth token used to authorize with the lima-city API.                                                                                           |                | yes                 |
| `DOMAIN_DATA` | String containing domain and record IDs that should be updated. Take a look at the [#formatting-your-domain_data](#formatting-your-domain_data) section. |                | yes                 |
| `MODE`        | Select the mode you want to use. It is either `router` or `external_api`. The modes are described in the [#docker-compose](#docker-compose) section.     | `external_api` | no                  |
| `PASSWORD`    | Password to authenticate in router mode. Must be URL encoded - [this website](https://www.urlencoder.org) worked perfectly for me.                    |                | only in router mode |
