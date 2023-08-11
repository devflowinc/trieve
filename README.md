<p align="center">
  <img height="100" src="https://raw.githubusercontent.com/arguflow/blog/5ef439020707b0e27bf901c8f6b4fb1f487a78d4/apps/frontend/public/assets/horizontal-logo.svg" alt="Arguflow">
</p>

<p align="center">
    <b>Easy to use abstraction over Qdrant and Postgres for creating a semantic/full-text socially enabled embedding store on your data</b>
</p>

**Arguflow Vault Server**: Paired with [vault-client](https://github.com/arguflow/vault-client), Arguflow Vault is an abstraction layer over Qdrant which provides semantic and full-text search over arbitrary HTML cards, WYSIWYG TinyMCE editor, collection based organization, MMR, voting, and more. Vault Server is written in Rust 🦀 with Node and Python services where appropriate. Vault is still in early alpha, but you may find it useful if you are trying to build semantic or full text search over your data.

<p align="center">
<strong><a href="https://docs.arguflow.ai">Documentation</a> • <a href="https://vault.arguflow.ai">Competitive Debate Demo</a> • <a href="https://discord.gg/CuJVfgZf54">Discord</a>

</strong>
</p>

# Vault Server

Server for providing both semantic and full text search, voting, and collection based organization.

This project utilizes [Qdrant](https://qdrant.tech/) and [actix-web](https://actix.rs), a [Rust](https://www.rust-lang.org) language framework.

## How to contribute

1. Fork the repository and clone it to your local machine
2. Create a new branch with a descriptive name: git checkout -b your-branch-name
3. Make your changes to the README file. Please ensure that your changes are relevant and add value to the project
4. Test your changes locally to ensure that they do not break anything
5. Commit your changes with a descriptive commit message: git commit -m "Add descriptive commit message here"
6. Push your changes to your forked repository: git push origin your-branch-name
7. Open a pull request to the main repository and describe your changes in the PR description

## Storing environment variables in .env file

Create a .env file in the root directory of the project. This .env file will require the following url's and API keys

`SALT`, `SECRET_KEY`, and the `STRIPE` keys are all optional.

```
DATABASE_URL=postgresql://postgres:password@localhost:5432/vault
REDIS_URL=redis://localhost:6379
QDRANT_URL=http://127.0.0.1:6334
SENDGRID_API_KEY=*******************
OPENAI_API_KEY=*******************
STRIPE_API_SECRET_KEY=*******************
STRIPE_SILVER_PLAN_ID=*******************
STRIPE_GOLD_PLAN_ID=*******************
WEBHOOK_SIGNING_SECRET=*******************
SECRET_KEY=*******************
SALT=*******************
LIBREOFFICE_PATH=libreoffice
S3_ENDPOINT=*******************
S3_ACCESS_KEY=*******************
S3_SECRET_KEY=*******************
S3_BUCKET=vault
VERIFICATION_SERVER_URL=http://localhost:8091/get_url_content
QDRANT_API_KEY=qdrant_pass
COOKIE_SECURE=false
ADMIN_UUID=********************
```

## Getting started

The following information is also automated via the Dockerfile. The quick start would be:

```
docker build -t vault-server .
docker run -p 8090:8090 vault-server
```

then follow the S3 instructions. Your `.env` file also needs to be renamed to `.env.docker`.

### Install apt packages

```
curl \
gcc \
g++ \
make \
pkg-config \
python3 \
python3-pip \
libpq-dev \
libssl-dev \
openssl \
libreoffice
```

### Install NodeJS and Yarn

You can use the following, but we recommend using [NVM](https://github.com/nvm-sh/nvm) and then running `yarn --cwd ./vault-nodejs install` .

```
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && \
    apt-get install -y nodejs && \
    npm install -g yarn && \
    yarn --cwd ./vault-nodejs install
```

### Set rust to nightly

`rustup default nightly`

### Make admin account

Use [vault-client](https://github.com/arguflow/vault-client) to make an account. Get its uuid and set `ADMIN_UUID` to equal it in your .env

### Install python requirements

`pip install -r ./vault-python/requirements.txt`

### Setting Up Local S3

1. `sudo docker compose up s3`
2. Go to the MinIO dashboard at [http://127.0.0.1:42625](http://127.0.0.1:42625)
3. Sign in with `rootuser` and `rootpassword`
4. Go to [http://127.0.0.1:42625/identity/users](http://127.0.0.1:42625/identity/users)
5. Set `S3_ENDPOINT` to `http://127.0.0.1:9000`
6. Click "Create User"
7. Set the credentials to "s3user" and "s3password"
8. Assign the user roles for access
9. Click the user and navigate to "Service Accounts"
10. Click "Create Access Key" and save the created keys to your .env under `S3_ACCESS_KEY` and `S3_SECRET_KEY` respectively
11. Click "Buckets" and then "Create Bucket"
12. Create a bucket named `vault` and set `S3_BUCKET` in the env to `vault`

### Preparing for file uploads and conversions

1. `sudo apt install pandoc`
2. `mkdir tmp` from inside repository folder

If you want to test things outside of the minio dashboard and Tokio server, then setup AWS CLI using [this guide from minio](https://min.io/docs/minio/linux/integrations/aws-cli-with-minio.html) with the region set to `""`

- You can install it with `sudo apt install awscli` assuming you are using the apt package manager

### Run the server in dev mode

```
docker compose up -d
cargo watch -x run
```

## Running the test suite

This section refers to the jest testing suite found in the `vault-nodejs` folder of this repository

1. Set the variables found in `.env.dist` in the `.env` file of the testing suite
2. `cd` into the testing suite folder
3. Run `yarn`
4. Run `yarn test`

## Resetting the application data db's

1. `sudo docker compose stop qdrant-database`
2. `sudo docker compose rm -f qdrant-database`
3. `sudo docker volume rm vault_qdrant_data`
4. `diesel db reset`

## Resetting the script db

1. `sudo docker compose stop script-redis`
2. `sudo docker compose rm -f script-redis`
3. `sudo docker volume rm vault_script-redis-data`

## How to debug diesel by getting the exact generated SQL

diesel::debug*query::<diesel::pg::Pg, *>(&query);

## How to set up the python verification script

1. `virtualenv venv`
2. `source venv/bin/activate`
3. `pip install -r ./vault-python/requirements.txt`

## How to get Rust debug level logs

Run `export RUST_LOG=debug`
