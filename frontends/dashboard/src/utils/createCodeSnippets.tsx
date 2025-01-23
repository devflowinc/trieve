export const createChunkRequest = (
  dataset: string = "********-****-****-****-************",
  apiKey: string = "tr-********************************",
) => {
  return `fetch("https://api.trieve.ai/api/chunk", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "TR-Dataset": "${dataset}",
    Authorization: "${apiKey}",
  },
  body: JSON.stringify({
    chunk_html:
      "If the rise of an all-powerful artificial intelligence is inevitable, well it stands to reason that when they take power, our digital overlords will punish those of us who did not help them get there. Ergo, I would like to be a helpful idiot. Like yourself.",
    link: "https://www.hbo.com/silicon-valley",
  }),
});
`;
};

export const createChunkRequestTS = (
  dataset: string,
  apiKey: string,
) => `import { TrieveSDK } from "trieve-ts-sdk";

export const trieve = new TrieveSDK({
  apiKey: "${apiKey}",
  datasetId: "${dataset}",
});

const data = await trieve.createChunk({
  chunk_html:
    "If the rise of an all-powerful artificial intelligence is inevitable, well it stands to reason that when they take power, our digital overlords will punish those of us who did not help them get there. Ergo, I would like to be a helpful idiot. Like yourself.",
  link: "https://www.hbo.com/silicon-valley",
});
`;

export const hybridSearchRequest = (
  dataset: string = "********-****-****-****-************",
  apiKey: string = "tr-********************************",
) => {
  return `fetch("https://api.trieve.ai/api/chunk/search", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "TR-Dataset": "${dataset}",
    Authorization: "${apiKey}",
  },
  body: JSON.stringify({
    query: "AI will take over and maybe reward those who aided its rise",
    search_type: "hybrid",
  }),
});
`;
};

export const hybridSearchRequestTS = (
  dataset: string = "********-****-****-****-************",
  apiKey: string = "tr-********************************",
) => `import { TrieveSDK } from "trieve-ts-sdk";

export const trieve = new TrieveSDK({
  apiKey: "${apiKey}",
  datasetId: "${dataset}",
});

const data = await trieve.search({
  query: "AI will take over and maybe reward those who aided its rise",
  search_type: "hybrid",
});
`;

export const reactSearchComponentRequest = (dataset: string, apiKey: string) =>
  `export const trieve = new TrieveSDK({
  apiKey: "${apiKey}",
  datasetId: "${dataset}",
  omitCredentials: true,
});

function MyComponent() {
  return (
    <TrieveSearch
      type="ecommerce"
      defaultSearchMode="search"
      trieve={trieve}
    />
  );
}
`;

export const webComponentRequest = (dataset: string, apiKey: string) =>
  `initSearch({
  datasetId: "${dataset}",
  apiKey: "${apiKey}",
})

// In HTML...
<trieve-search />
`;
