# PassPhraseX API

## How to use
### Locally
- Clone the repository
- Change directory to `api`
- Run `cargo run -r`
- The api will be running on `localhost:3000`

### On Server
- Clone the repository
- Change directory to `api`
- Modify the `nginx/nginx.conf` file to use your own domain
- Modify the `compose.yaml` file to use your own domain & email
- Install docker & docker compose
- Run `docker-compose up -d`
- The compose file will build the image and run the container 