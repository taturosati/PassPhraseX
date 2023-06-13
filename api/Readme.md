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
- Install docker & docker compose
- Run `docker-compose up -d`
- The compose file will build the image and run the container
    - This includes mongodb database, nginx reverse proxy (with lets encrypt) and the api
    - You need to modify compose.yaml to use your own domain & email
    - You also need to modify nginx.conf to use your own domain
    - Replace api.passphrasex.srosati.xyz with your own domain