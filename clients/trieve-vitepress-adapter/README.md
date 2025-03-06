# Trieve Vitepress adapter

The easiest way to use Trieve in combination with Vitepress.

## Collect all the data

### Trieve

You need to retrieve your `Org ID` and a new `API Key`:

1. Go to the [Trieve dashboard](https://dashboard.trieve.ai/org).
2. Copy the `Org ID`, you will need it later.
3. Press on the `API Keys` tab.
4. Press `Create New Key` and select `OWNER` for the permission.
5. Copy the `API Key`, you will need it later.

### Documents

You need:

- the path to your `<docs.yml>`, e.g. `.`.
- (optional): the url to your `<openapi.json>`, e.g. `https://api.vapi.ai/api-json`.
- (optional): the `<root url>` to your docs, e.g. `https://docs.vapi.ai`.
- (optional): the `<api-reference-path>`, e.g. `api-reference`.

## Local Setup

### Installation

```bash
npm install -g trieve-vitepress-adapter
```

### Environment

You need these environment variables set:

```
TRIEVE_API_HOST=https://api.trieve.ai
TRIEVE_API_KEY=
TRIEVE_ORGANIZATION_ID=
TRIEVE_DATASET_TRACKING_ID=
```

The `TRIEVE_DATASET_TRACKING_ID` must be an unique identifier for the dataset, e.g. `vapi`.

### Execution

```bash
trieve-vitepress-adapter --file <docs.yml> -s <openapi.json> -r <root-url> -a <api-reference-page>
```

## CI Setup (GitHub)

### Environment

Set these repository secrets:

```
TRIEVE_API_HOST=https://api.trieve.ai
TRIEVE_API_KEY=
TRIEVE_ORGANIZATION_ID=
TRIEVE_DATASET_TRACKING_ID=
```

### Workflow

Add this workflow to `.github/workflows`.

```yml
name: Update Trieve

concurrency:
  group: ${{ github.workflow }}-${{ github.head_ref }}
  cancel-in-progress: true

on:
  push:
    branches:
      - main

jobs:
  run:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Trieve Vitepress Adapter
        run: npm install -g trieve-vitepress-adapter

      - name: Update Trieve Chunks 
        env:
          TRIEVE_API_HOST: ${{ secrets.TRIEVE_API_HOST }}
          TRIEVE_API_KEY: ${{ secrets.TRIEVE_API_KEY }}
          TRIEVE_ORGANIZATION_ID: ${{ secrets.TRIEVE_ORGANIZATION_ID }}
          TRIEVE_DATASET_TRACKING_ID: ${{ secrets.TRIEVE_DATASET_TRACKING_ID }}
        run: trieve-vitepress-adapter --file <docs.yml> -s <openapi.json> -r <root-url> -a <api-reference-page>

```

Replace `<docs.yml>`, `<openapi.json>`, `<root-url>`, and `<api-reference-page>`.

## License

MIT
