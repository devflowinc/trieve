## Trieve Search Component

The easiest way to get up and running in your app using trieve search.

## How to use

Install using your favorite package manager:

```
yarn add trieve-search-component
# or
npm install trieve-search-component
# or
pnpm install trieve-search-component
```

After installing the first step is to instantiate a new `TrieveSDK` like so:

```ts
import { TrieveSDK } from "trieve-ts-sdk";

export const trieve = new TrieveSDK({
  apiKey: "<your-api-key>",
  datasetId: "<dataset-to-use>",
});
```

And then you can use any of the two components in your React application or as web component:

### Search Modal

<details>
<summary>Screenshots</summary>

![light closed](./github/modal-light-1.png)
![dark closed](./github/modal-dark-1.png)
![light open](./github/modal-light-2.png)

</details>

#### Usage in React:

```jsx
<TrieveModalSearch trieve={trieve} />
```

#### Usage in Web Components:

```js
initModalSearch({
  trieve: new TrieveSDK({
    // your options
  })
})


<trieve-modal-search />

```

If you are using it in JSX environment you will need to add the `trieve-modal-search` to the JSX attributes, for solid that would be:

```typescript
declare module "solid-js" {
  namespace JSX {
    interface IntrinsicElements {
      "trieve-modal-search": {};
      "trieve-search": {};
    }
  }
}
```

#### Props

| Name          | Type                                                                                           | Default                     |
| ------------- | ---------------------------------------------------------------------------------------------- | --------------------------- |
| trieve        | TrieveSDK                                                                                      | null                        |
| chat          | boolean                                                                                        | true                        |
| analytics     | boolean                                                                                        | true                        |
| showImages    | boolean                                                                                        | false                       |
| placeholder   | string                                                                                         | "Search..."                 |
| onResultClick | () => void                                                                                     | () => {}                    |
| theme         | "light" or "dark"                                                                              | "light"                     |
| searchOptions | [SearchChunksReqPayload](https://ts-sdk.trieve.ai/types/types_gen.SearchChunksReqPayload.html) | `{ search_type: "hybrid" }` |

### Search Results

<details>
<summary>Screenshots</summary>

![light](./github/search-light.png)
![dark](./github/search-dark.png)

</details>

#### Usage in React:

```jsx
<TrieveSearch trieve={trieve} />
```

#### Usage in Web Components:

```html
<trieve-search trieve="<your trieve instance>" />
```

#### Props

| Name          | Type                                                                                           | Default                     |
| ------------- | ---------------------------------------------------------------------------------------------- | --------------------------- |
| trieve        | TrieveSDK                                                                                      | null                        |
| showImages    | boolean                                                                                        | false                       |
| placeholder   | string                                                                                         | "Search..."                 |
| onResultClick | () => void                                                                                     | () => {}                    |
| theme         | "light" or "dark"                                                                              | "light"                     |
| searchOptions | [SearchChunksReqPayload](https://ts-sdk.trieve.ai/types/types_gen.SearchChunksReqPayload.html) | `{ search_type: "hybrid" }` |

## License

MIT
