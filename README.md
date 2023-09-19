<p align="center">
  <img height="100" src="https://raw.githubusercontent.com/arguflow/blog/5ef439020707b0e27bf901c8f6b4fb1f487a78d4/apps/frontend/public/assets/horizontal-logo.svg" alt="Arguflow">
</p>

<p align="center">
    <b>Software suite for deploying semantic search and RAG/no-hallucination LLM-chat over arbitrary data cards/chunks. Contains a server for duplication detection, bookmarks, retrieval, filtering, recommendations, etc. and also white-label'able UI's for search and retrieval-augmented LLM chat. Build, contribute, and stay-tuned!</b>
</p>
<p align="center">
<strong><a href="https://docs.arguflow.ai">Documentation</a> • <a href="https://search.arguflow.ai">Debate Search Demo</a> • <a href="https://chat.arguflow.ai">RAG Debate Opponent Demo</a> • <a href="https://discord.gg/CuJVfgZf54">Discord</a> • <a href="https://matrix.to/#/#arguflow-general:matrix.zerodao.gg">Matrix</a>

</strong>
</p>

# Arguflow

## How to contribute

1. Fork the repository and clone it to your local machine
2. Create a new branch with a descriptive name: git checkout -b your-branch-name
3. Make your changes to the README file. Please ensure that your changes are relevant and add value to the project
4. Test your changes locally to ensure that they do not break anything
5. Commit your changes with a descriptive commit message: git commit -m "Add descriptive commit message here"
6. Push your changes to your forked repository: git push origin your-branch-name
7. Open a pull request to the main repository and describe your changes in the PR description

## Self-hosting the API and UI's

We have a full self-hosting guide available on our [documentation page here](https://docs.arguflow.ai/self_hosting).

## Local development

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

### Install python requirements

`pip install -r ./server/vault-python/requirements.txt`

## How to debug diesel by getting the exact generated SQL

diesel::debug*query::<diesel::pg::Pg, *>(&query);

## How to set up the python verification script

1. `virtualenv venv`
2. `source venv/bin/activate`
3. `pip install -r ./server/vault-python/requirements.txt`

## How to get Rust debug level logs

Run `export RUST_LOG=debug`

## Recommended local dev setup

This repository used to solely house the `server` folder, but has recently been expanded to contain both `search` and `chat`. We recommend that you open VSCode for the `search` and `chat` folders independently. 

### Setup env's

```
cp .env.chat ./chat/.env
cp .env.search ./search/.env
cp ./server/.env.dist ./server/.env
```

### Start docker container services needed for local dev

```
./convenience.sh -l
```

### Start services for local dev

We know this is bad. Currently, We recommend managing this through tmux. 

```
cd server
cargo watch -x run
```

```
cd search
yarn
yarn dev
```

```
cd chat
yarn
yarn dev
```