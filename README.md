# hum

A [warp](https://github.com/seanmonstar/warp) & [reqwest](https://github.com/seanmonstar/reqwest) server to trigger Builds in the [Drone API](https://docs.drone.io/api/overview/) from a webhook endpoint.

## Running

With logging output:

```bash
RUST_LOG=info cargo run
```

To hack on it ([cargo-watch](https://github.com/passcod/cargo-watch) recommended):

```bash
cargo watch -x run
```

Run/Deploy via docker
```bash
# Build
docker build --rm -t hum .

# Run with Inline ENV vars
docker run --rm -d -p 3030 \
    -e DRONE_SERVER=https://your.drone.server \
    -e DRONE_TOKEN=yourDroneToken \
    --name hum hum

# Or Run with ENV var file

docker run --rm -d -p 3030 --env-file ./.env --env-file .env --name hum hum
```

## Configuration

Add your [Drone Server & Token](https://docs.drone.io/api/overview/) as well as address and port you want Hum to bind to via ENV Vars

```env
# .env
DRONE_SERVER=https://your.drone.server
DRONE_TOKEN=yourDroneToken
HUM_ADDR=0.0.0.0
HUM_PORT=3030
```

Go to the service that needs the webhook and point it at this server

`GET https://url.to.this.hum.server/<user>/<repo>`

or

`GET https://url.to.this.hum.server/<user>/<repo>?cancel_running=true`

Loosely based on [rust-reqwest-warp-example](https://github.com/halfzebra/rust-reqwest-warp-example)
