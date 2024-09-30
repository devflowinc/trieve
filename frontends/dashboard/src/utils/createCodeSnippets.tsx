export const createChunkRequest = (
  dataset: string = "********-****-****-****-************",
) => {
  return `fetch("https://api.trieve.ai/api/chunk", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "TR-Dataset": "${dataset}",
    Authorization: "tr-********************************",
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
) => `import { TrieveSDK } from "trieve-ts-sdk";

export const trieve = new TrieveSDK({
  apiKey: "tr-********************************",
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
) => {
  return `fetch("https://api.trieve.ai/api/chunk/search", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "TR-Dataset": "${dataset}",
    Authorization: "tr-********************************",
  },
  body: JSON.stringify({
    content: "AI will take over and maybe reward those who aided its rise",
    search_type: "hybrid",
  }),
});
`;
};

export const hybridSearchRequestTS = (
  dataset: string,
) => `import { TrieveSDK } from "trieve-ts-sdk";

export const trieve = new TrieveSDK({
  apiKey: "tr-********************************",
  datasetId: "${dataset}",
});

const data = await trieve.search({
  query: "AI will take over and maybe reward those who aided its rise",
  search_type: "hybrid",
});
`;
