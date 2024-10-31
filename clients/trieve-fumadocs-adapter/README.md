## Adding into your fumadocs site

Install dependencies

```sh
npm install trieve-fumadocs-adapter trieve-ts-sdk
```

### Local Development Guide

#### Run component Build script

```sh
clients/trieve-fumadocs-adapter $ npm run bundle
```

#### Run Example Fumadocs application
Link NPM package

```sh
clients/trieve-fumadocs-adapter $ npm link
```

Link package into example
```sh
clients/trieve-fumadocs-adapter/example $ npm link trieve-fumadocs-adapter
```

Run build script to update index
```sh
clients/trieve-fumadocs-adapter/example $ npm run build
```

Run example
```sh
clients/trieve-fumadocs-adapter/example $ npm run dev
```

### Publishing

DO NOT RUN `yarn publish`, instead run yarn pub
