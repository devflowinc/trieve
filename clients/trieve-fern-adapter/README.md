# Trieve Fern adapter

The easiest way to use Trieve in combination with Fern.

## Installation

```bash
npm install -g trieve-fern-adapter
```

## Setup

You need these environment variables set:

```
TRIEVE_API_HOST=https://api.trieve.ai
TRIEVE_API_KEY=
TRIEVE_ORGANIZATION_ID=
TRIEVE_DATASET_TRACKING_ID=
```

## How to use

```bash
trieve-fern-adapter --file <path-to-docs.yml> -s <openapi-spec-url> -r <root-url> -a <api-reference-page>
```

## License

MIT
