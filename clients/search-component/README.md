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

```html
<trieve-modal-search trieve="<your trieve instance>" />
```

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

## License

MIT
