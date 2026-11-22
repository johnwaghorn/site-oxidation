# Running on Unraid

There are two ways to run Site Oxidation on Unraid:

- From Community Apps, using the container template (recommended)
- With the Docker Compose Manager plugin

Both use the same image, `ghcr.io/johnwaghorn/site-oxidation`, and keep their
data in `/mnt/user/appdata/site-oxidation`.

## Install from Community Apps

1. Open the Apps tab and search for Site Oxidation.
2. Click Install. The defaults work for most setups.
3. Click Apply, then open the WebUI from the container's menu on the Docker tab.
4. Follow the setup wizard to create the first admin account.

The app is then at `http://<unraid-ip>:8026`.

### Install the template by hand

Until the app is listed in Community Apps, or to try a newer template, copy
[`unraid/site-oxidation.xml`](../../unraid/site-oxidation.xml) onto the server:

```bash
scp unraid/site-oxidation.xml root@<unraid-ip>:/boot/config/plugins/dockerMan/templates-user/my-site-oxidation.xml
```

Then go to Docker, Add Container, and pick `site-oxidation` from the Template
list under User templates.

### Template settings

| Setting | Default | What it does |
|---|---|---|
| Web UI port | `8026` | Port you browse to. Change it if 8026 is already taken. |
| App data | `/mnt/user/appdata/site-oxidation` | Database and session key. Back this folder up. |
| `COOKIE_SECURE` | `false` | See [HTTPS](#https-with-a-reverse-proxy) below. |
| `PROBE_ALLOW_PRIVATE_IPS` | `false` | Set to `true` to monitor services on your own network. |

Turn on Advanced View to see the rest:

| Setting | Default | What it does |
|---|---|---|
| `SMTP_ALLOW_PRIVATE_HOSTS` | `false` | Set to `true` to send email alerts through a mail server on your network. |
| `CERT_WARN_DAYS` | `30` | Days before expiry that a certificate shows as expiring soon. |
| `CERT_CRITICAL_DAYS` | `7` | Days before expiry that a certificate shows as critical. |
| `RUST_LOG` | `info` | Set to `debug` when reporting a problem. |
| `PUID` / `PGID` | `99` / `100` | User and group the app runs as (Unraid's `nobody:users`). |

Any other [environment variable](../environment_variables.md) can be added with
"Add another Path, Port, Variable, Label or Device" on the container's edit
page.

## Install with Docker Compose

This needs the Docker Compose Manager plugin from Community Apps.

1. On the Docker tab, under Compose, click Add New Stack and name it
   `site-oxidation`.
2. Edit the stack's compose file and paste in
   [`unraid/docker-compose.yml`](../../unraid/docker-compose.yml).
3. Edit the stack's env file and paste in
   [`unraid/.env.example`](../../unraid/.env.example), changing any values you
   need.
4. Click Compose Up.

The app is then at `http://<unraid-ip>:8026`.

Only run one of the two installs at a time. Both use the same container name,
port and data folder, and two copies would both run checks and write to the same database.

## HTTPS with a reverse proxy

Site Oxidation serves plain HTTP. For HTTPS, put it behind the reverse proxy you
already use, such as SWAG, Nginx Proxy Manager or a Cloudflare Tunnel. Point the
proxy at `http://<unraid-ip>:8026`. No special proxy settings are needed.

Then set `COOKIE_SECURE` to `true` and restart the container. This stops the
browser sending the login cookie over plain HTTP.

| How you open the app | `COOKIE_SECURE` |
|---|---|
| `http://<unraid-ip>:8026` on your LAN | `false` |
| `https://` through a reverse proxy | `true` |

With `COOKIE_SECURE=true`, logging in over plain HTTP silently fails... the login
appears to work, then you are sent straight back to the login page. Use the
HTTPS address only. If the app is reachable from the internet, it must be
`true`.

## Updates and backups

Updates arrive as new images. Use Check for Updates on the Docker tab, or
Compose Pull and Compose Up for the Compose install. Your sites, history and
users are kept in the app data folder.

To back up, copy the app data folder. It holds:

- `site-oxidation.db`, the database
- `session.key`, which signs login cookies

> [!NOTE]
> Losing `session.key` signs everyone out but loses no data. A new one is
> created on the next start.

## Troubleshooting

| Problem | Fix |
|---|---|
| Login does not stick | `COOKIE_SECURE` is `true` but you are using plain HTTP. Set it to `false`, or use the HTTPS address. |
| A site on your network is refused | Set `PROBE_ALLOW_PRIVATE_IPS` to `true`. |
| Email through a local mail server is refused | Set `SMTP_ALLOW_PRIVATE_HOSTS` to `true`. |
| The container stops straight after starting | Check its log on the Docker tab. An invalid setting is named in the error. |
| Need more detail for a bug report | Set `RUST_LOG` to `debug`, restart, and include the log. |

Report problems on
[GitHub issues](https://github.com/johnwaghorn/site-oxidation/issues).
