import {
  ChunkMetadata,
  SearchChunksReqPayload,
} from "trieve-ts-sdk";

export type Chunk = Omit<ChunkMetadata, "metadata"> & {
  highlight?: string | undefined | null;
  highlightTitle?: string | undefined | null;
  highlightDescription?: string | undefined | null;
  metadata: {
    [key: string]: string;
  };
};

export type ChunkWithHighlights = { chunk: Chunk; highlights: string[] };

export type SearchResults = {
  chunks: ChunkWithHighlights[];
  requestID: string;
};

export type Props = {
  datasetId: string;
  apiKey: string;
  onResultClick?: (chunk: Chunk, requestID: string) => void;
  theme?: "light" | "dark";
  searchOptions?: Omit<
    Omit<SearchChunksReqPayload, "query">,
    "highlight_options"
  >;
  placeholder?: string;
};
