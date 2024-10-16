## Adding into your site

Add the following into your `docusaurus.config.js`

```sh
npm install @trieve/docusaurus-search-theme
```

```js docusaurus.config.js
/** @type {import('@docusaurus/types').Config} */
const config = {
  ...
  themes: [
    [
      require.resolve("@trieve/docusaurus-search-theme"),
      {
        apiKey: "tr-****************",
        datasetId: "<your-dataset-id>"
        defaultSearchQueries: [
          "How to use the API reference?",
          "Is there a python sdk?",
          "How to get started?"
        ]
      }
    ],
  ],
};
```

## Configuration options available

| Name                 | Type                                                                                           | Default                                    |
| -------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------ |
| datasetId            | string                                                                                         | ''                                         |
| apiKey               | string                                                                                         | ''                                         |
| chat                 | boolean                                                                                        | true                                       |
| analytics            | boolean                                                                                        | true                                       |
| placeholder          | string                                                                                         | "Search..."                                |
| theme                | "light" or "dark"                                                                              | "light"                                    |
| searchOptions        | [AutocompleteReqPayload](https://ts-sdk.trieve.ai/types/types_gen.AutocompleteReqPayload.html) | `{ search_type: "fulltext" }`              |
| suggestedQueries     | boolean                                                                                        | true                                       |
| defaultSearchQueries | string[]                                                                                       | []                                         |
| defaultAiQuestions   | string[]                                                                                       | []                                         |
| brandLogoImgSrcUrl   | string                                                                                         | null                                       |
| brandName            | string                                                                                         | null                                       |
| accentColor          | string                                                                                         | #CB53EB                                    |
