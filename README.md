<p align="center">
  <img height="100" src="https://cdn.trieve.ai/trieve-logo.png" alt="Trieve Logo">
</p>
<p align="center">
<strong><a href="https://dashboard.trieve.ai">Sign Up (1k chunks free)</a> | <a href="https://docs.trieve.ai">Documentation</a> | <a href="https://cal.com/nick.k/meet">Meeting Link</a> | <a href="https://discord.gg/eBJXXZDB8z">Discord</a> | <a href="https://matrix.to/#/#trieve-general:trieve.ai">Matrix</a>
</strong>
</p>

<p align="center">
    <a href="https://github.com/devflowinc/trieve/stargazers">
        <img src="https://img.shields.io/github/stars/devflowinc/trieve.svg?style=flat&color=yellow" alt="Github stars"/>
    </a>
    <a href="https://github.com/devflowinc/trieve/issues">
        <img src="https://img.shields.io/github/issues/devflowinc/trieve.svg?style=flat&color=success" alt="GitHub issues"/>
    </a>
    <a href="https://discord.gg/CuJVfgZf54">
        <img src="https://img.shields.io/discord/1130153053056684123.svg?label=Discord&logo=Discord&colorB=7289da&style=flat" alt="Join Discord"/>
    </a>
    <a href="https://matrix.to/#/#trieve-general:trieve.ai">
        <img src="https://img.shields.io/badge/matrix-join-purple?style=flat&logo=matrix&logocolor=white" alt="Join Matrix"/>
    </a>
</p>

<h2 align="center">
    <b>Trieve is all-in-one infrastructure for building hybrid vector search, recommendations, and RAG</b>
</h2>

![Trieve OG tag](https://cdn.trieve.ai/trieve-og.png)

## Quick Links

- [Why Search Before Generate](https://docs.trieve.ai/why_search_before_generate)
- [API Documentation](https://docs.trieve.ai)
- [OpenAPI specification](https://api.trieve.ai/redoc)

## Features

- **🔒 Self-Hosting in your VPC or on-prem**: Buy a license to host in your company's VPC on prem with our ready-to-go docker containers and terraform templates.
- **🧠 Semantic Dense Vector Search**: Integrates with OpenAI or Jina embedding models and [Qdrant](https://qdrant.tech) to provide semantic vector search.
- **🔍 Typo Tolerant Full-Text/Neural Search**: Every uploaded chunk is vector'ized with [naver/efficient-splade-VI-BT-large-query](https://huggingface.co/naver/efficient-splade-VI-BT-large-query) for typo tolerant, quality neural sparse-vector search.
- **🖊️ Sub-Sentence Highlighting**: Highlight the matching words or sentences within a chunk and bold them on search to enhance UX for your users. Shout out to the [simsearch](https://github.com/smartdatalake/simsearch) crate!
- **🌟 Recommendations**: Find similar chunks (or files if using grouping) with the recommendation API. Very helpful if you have a platform where users favorite, bookmark, or upvote content.
- **🤖 Convenient RAG API Routes**: We integrate with OpenRouter to provide you with access to any LLM you would like for RAG. Try our routes for [fully-managed RAG with topic-based memory management](https://api.trieve.ai/redoc#tag/message/operation/create_message_completion_handler) or [select your own context RAG](https://api.trieve.ai/redoc#tag/chunk/operation/generate_off_chunks).
- **💼 Bring Your Own Models**: If you'd like, you can bring your own text-embedding, SPLADE, cross-encoder re-ranking, and/or large-language model (LLM) and plug it into our infrastructure.
- **🔄 Hybrid Search with cross-encoder re-ranking**: For the best results, use hybrid search with [BAAI/bge-reranker-large](https://huggingface.co/BAAI/bge-reranker-large) re-rank optimization.
- **📆 Recency Biasing**: Easily bias search results for what was most recent to prevent staleness
- **🛠️ Tunable Popularity-Based Ranking (Merchandizing)**: Weight indexed documents by popularity, total sales, or any other arbitrary metric for tunable relevancy
- **🕳️ Filtering**: Date-range, substring match, tag, numeric, and other filter types are supported.
- **🧐 Duplicate Detection**: Check out our docs on [collision-based dup detection](https://docs.trieve.ai/duplicate_detection) to learn about how we handle duplicates. This is a setting you can turn on or off.
- **👥 Grouping**: Mark multiple chunks as being part of the same file and search on the file-level such that the same top-level result never appears twice

**Are we missing a feature that your use case would need?** - call us at [628-222-4090](mailto:+16282224090), make a [Github issue](https://github.com/devflowinc/trieve/issues), or join the [Matrix community](https://matrix.to/#/#trieve-general:trieve.ai) and tell us! We are a small company who is still very hands-on and eager to build what you need; professional services are available.

## Roadmap

Our current top 2 priorities for the next while are as follows. **Subject to change as current or potential customers ask for things.**

1. Observability and metrics (likely something w/ Clickhouse)
2. Benchmarking (going to aim for a 1M, 10M, and 100M vector benchmark)
3. SDKs (can generate from OpenAPI spec, but would like to test a bit more)

## How to contribute

1. Find an issue in the [issues tab](https://github.com/devflowinc/trieve/issues) that you would like to work on.
2. Fork the repository and clone it to your local machine
3. Create a new branch with a descriptive name: git checkout -b your-branch-name
4. Solve the issue by adding or removing code on your forked branch.
5. Test your changes locally to ensure that they do not break anything
6. Commit your changes with a descriptive commit message: git commit -m "Add descriptive commit message here"
7. Push your changes to your forked repository: git push origin your-branch-name
8. Open a pull request to the main repository and describe your changes in the PR description

## Self-hosting the API and UI's

We have a full self-hosting guide available on our [documentation page here](https://docs.trieve.ai/self_hosting).

## Local development with Linux

### Debian/Ubuntu Packages needed packages

```sh
sudo apt install curl \
gcc \
g++ \
make \
pkg-config \
python3 \
python3-pip \
libpq-dev \
libssl-dev \
openssl
```

### Arch Packages needed

```sh
sudo pacman -S base-devel postgresql-libs
```

### Install NodeJS and Yarn

You can install [NVM](https://github.com/nvm-sh/nvm) using its install script.

```
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.5/install.sh | bash
```

You should restart the terminal to update bash profile with NVM. Then, you can install NodeJS LTS release and Yarn.

```
nvm install --lts
npm install -g yarn
```

### Make server tmp dir

```
mkdir server/tmp
```

### Install rust

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install cargo-watch

```
cargo install cargo-watch
```

### Setup env's

```
cp .env.analytics ./frontends/analytics/.env
cp .env.chat ./frontends/chat/.env
cp .env.search ./frontends/search/.env
cp .env.server ./server/.env
cp .env.dashboard ./frontends/dashboard/.env
```

### Add your `LLM_API_KEY` to `./server/.env`

[Here is a guide for acquiring that](https://blog.streamlit.io/beginners-guide-to-openai-api/#get-your-own-openai-api-key).

#### Steps once you have the key

1. Open the `./server/.env` file
2. Replace the value for `LLM_API_KEY` to be your own OpenAI API key.
3. Replace the value for `OPENAI_API_KEY` to be your own OpenAI API key.

### Start docker container services needed for local dev

```
cat .env.chat .env.search .env.server .env.docker-compose > .env

./convenience.sh -l
# or
COMPOSE_PROFILE=dev docker compose up
```

### Start services for local dev

We know this is bad. Currently, we recommend managing this through [tmux](https://gist.github.com/skeptrunedev/101c7a13bb9b9242999830655470efac) or VSCode terminal tabs.

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

We have [tmux config](https://gist.github.com/skeptrunedev/101c7a13bb9b9242999830655470efac) we use internally you can use.

## Local development with Windows

### Install NodeJS and Yarn

You can download the latest version NodeJS from [here](https://nodejs.org/en/download). Open the downloaded file and follow the steps from the installer.

After completing the installation, open a powershell with administrator permissions.

```
npm install -g yarn
```

After installation, yarn might throw an error when used due to Window's execution policy. Change the execution policy to allow scripts to be executed by applications that are signed by a trusted publisher by putting this command in an admin powershell.

```
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned
```

### Install Rust

You can download the latest version of Rust from [here](https://www.rust-lang.org/tools/install). Follow the installer's directions and install the prerequisites.

After installation, open a new powershell window with administrator permissions.

```
cargo install cargo-watch
```

### Install Docker

Follow the instructions to download Docker Desktop for Windows from [here](https://docs.docker.com/desktop/install/windows-install/). You may need to follow the instructions to enable WSL 2.

### Install Postgres dependencies for building

Download PostgreSQL 13 from [here](https://www.enterprisedb.com/downloads/postgres-postgresql-downloads). You should not use any other version of PostgreSQL due to there being an [issue](https://github.com/diesel-rs/diesel/discussions/2947) with diesel on other versions.

When installing, ensure that the PostgreSQL server is set to a port other than 5432 to prevent it from interfering with the docker container.

Add Postgres to PATH

```
[Environment]::SetEnvironmentVariable("PATH", $Env:PATH + ";C:\Program Files\PostgreSQL\13\lib;C:\Program Files\PostgreSQL\13\bin", [EnvironmentVariableTarget]::Machine)
```

### Setup env's

```
cp .env.analytics ./frontends/analytics/.env
cp .env.chat ./frontends/chat/.env
cp .env.search ./frontends/search/.env
cp .env.server ./server/.env
cp .env.dashboard ./frontends/dashboard/.env
```

### Add your `LLM_API_KEY` to `./server/.env`

[Here is a guide for acquiring that](https://blog.streamlit.io/beginners-guide-to-openai-api/#get-your-own-openai-api-key).

#### Steps once you have the key

1. Open the `./server/.env` file
2. Replace the value for `LLM_API_KEY` to be your own OpenAI API key.
3. Replace the value for `OPENAI_API_KEY` to be your own OpenAI API key.

### Start Docker containers

Start the docker containers using the batch script.

```
Get-Content .env.chat, .env.search, .env.server, .env.docker-compose | Set-Content .env
./convenience.bat l
```

### Start services for local dev

You need 3 different windows of powershell or use something like VSCode terminal tabs to manage it.

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

## Install ImageMagick (Linux) - only needed if you want to use pdf_from_range route

```
apt install libjpeg-dev libpng-dev libtiff-dev

curl https://imagemagick.org/archive/ImageMagick.tar.gz | tar xz
cd ImageMagick
./configure
make uninstall
make install
```

## How to debug diesel by getting the exact generated SQL

`diesel::debug_query(&query).to_string();`

## Local Setup for Testing Stripe Features

Install Stripe CLI.

1. `stripe login`
2. `stripe listen --forward-to localhost:8090/api/stripe/webhook`
3. set the `STRIPE_WEBHOOK_SECRET` in the `server/.env` to the resulting webhook signing secret
4. `stripe products create --name trieve --default-price-data.unit-amount 1200 --default-price-data.currency usd`
5. `stripe plans create --amount=1200 --currency=usd --interval=month --product={id from response of step 3}`

## SelfHosting / Deploy to AWS

Refer to the self hosting guide [here](self-hosting.md)
