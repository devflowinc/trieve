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

| Name                 | Type                                                                                           | Default                                    |
| -------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------ |
| datasetId            | string                                                                                         | ''                                         |
| apiKey               | string                                                                                         | ''                                         |
| baseUrl              | string                                                                                         | "https://api.trieve.ai"                    |
| chat                 | boolean                                                                                        | true                                       |
| analytics            | boolean                                                                                        | true                                       |
| placeholder          | string                                                                                         | "Search..."                                |
| onResultClick        | () => void                                                                                     | () => {}                                   |
| theme                | "light" or "dark"                                                                              | "light"                                    |
| searchOptions        | [AutocompleteReqPayload](https://ts-sdk.trieve.ai/types/types_gen.AutocompleteReqPayload.html) | `{ search_type: "fulltext" }`              |
| openKeyCombination   | { key?: string; label?: string; ctrl?: boolean }[]                                             | [{ ctrl: true }, { key: "k", label: "K" }] |
| ButtonEl             | JSX.ElementType                                                                                | null                                       |
| suggestedQueries     | boolean                                                                                        | true                                       |
| defaultSearchQueries | string[]                                                                                       | []                                         |
| defaultAiQuestions   | string[]                                                                                       | []                                         |
| brandLogoImgSrcUrl   | string                                                                                         | null                                       |
| brandName            | string                                                                                         | null                                       |
| brand Color          | string                                                                                         | #CB53EB                                    |
| problemLink          | string (example: "mailto:help@trieve.ai?subject=")                                             | null                                       |
| responsive           | boolean                                                                                        | false                                      |

### Search Results

<details>
<summary>Screenshots</summary>

![light](./github/search-light.png)
![dark](./github/search-dark.png)

</details>

#### Usage in React:

```jsx
<TrieveSearch apiKey="<your trieve apiKey>" datasetId="<your trieve datasetId" />
```

#### Usage in Web Components:

```html
<trieve-search apiKey="<your trieve apiKey>" datasetId="<your trieve datasetId" />
```

#### Usage with Vanilla JS
```javascript
    import { renderToDiv } from 'trieve-search-component';
    const root = document.getElementById('root');
    renderToDiv(root, {
      apiKey: "<Your Trieve Api Key>"
      datasetId: "<Your Trieve Dataset Id>"
       // ... other props
    })
```

#### Props

| Name          | Type                                                                                           | Default                     |
| ------------- | ---------------------------------------------------------------------------------------------- | --------------------------- |
| datasetId     | string                                                                                         | ''                          |
| apiKey        | string                                                                                         | ''                          |
| placeholder   | string                                                                                         | "Search..."                 |
| onResultClick | () => void                                                                                     | () => {}                    |
| theme         | "light" or "dark"                                                                              | "light"                     |
| searchOptions | [SearchChunksReqPayload](https://ts-sdk.trieve.ai/types/types_gen.SearchChunksReqPayload.html) | `{ search_type: "hybrid" }` |

## License

MIT

## Development Guide

The `example/` folder shows the example application for what rendering this would look like

Start the listener to update the search-component's css and javascript

```sh
$clients/search-component yarn
$clients/search-component yarn dev
```

Run the example application

```sh
$clients/search-component cd example/
$clients/search-component yarn
$clients/search-component yarn dev
```
