<p align="center">
  <img height="100" src="https://raw.githubusercontent.com/arguflow/blog/5ef439020707b0e27bf901c8f6b4fb1f487a78d4/apps/frontend/public/assets/horizontal-logo.svg" alt="Arguflow">
</p>

<p align="center">
    <b>Easy to use abstraction over Qdrant and Postgres for creating a semantic/full-text socially enabled embedding store on your data</b>
</p>

**Arguflow Vault Search**: Paired with [vault-server](https://github.com/arguflow/vault-server), Arguflow Vault is an abstraction layer over Qdrant which provides semantic and full-text search over arbitrary HTML cards, WYSIWYG TinyMCE editor, collection based organization, MMR, voting, and more. Vault Search is written in Astro and SolidJS. Vault is still in early alpha, but you may find it useful if you are trying to build semantic or full text search over your data.

<p align="center">
<strong><a href="https://docs.arguflow.ai">Documentation</a> • <a href="https://search.arguflow.ai">Competitive Debate Demo</a> • <a href="https://discord.gg/CuJVfgZf54">Discord</a>

</strong>
</p>

# Vault Search 

## Getting Started 

### Set your .env

```
API_HOST=http://127.0.0.1:8090/api
PUBLIC_API_HOST=http://localhost:8090/api

# API_HOST=https://api.arguflow.ai/api
# PUBLIC_API_HOST=https://api.arguflow.ai/api

PUBLIC_HOST=http://localhost:8090
PLAUSIBLE_HOST=**********
```

### Run the client in dev mode

```
yarn
yarn dev 
```

### Run with docker

```
docker build -t vault-search .
docker run -p 8090:8090 vault-search
```