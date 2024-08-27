A Typescript based SDK that allows you to communicate with the Trieve API

## How to use

Install using your favorite package manager:

```
yarn add trieve-ts-sdk
# or
npm install trieve-ts-sdk
# or
pnpm install trieve-ts-sdk
```

After installing the first step is to instantiate a new `TrieveSDK` like so:

```ts
import { TrieveSDK } from "trieve-ts-sdk";

export const trieve = new TrieveSDK({
  apiKey: "<your-api-key>",
  datasetId: "<dataset-to-use>",
});
```

With this done you can use any of the functions available to use trieve in your project and get searching:

```ts
import { trieve } from "../trieve";

const data = await trieve.search({
  query: "my first query",
  search_type: "hybrid",
});
```

To see all the functions we export you can take a look at [our docs](https://ts-sdk.trieve.ai).

## License

MIT
