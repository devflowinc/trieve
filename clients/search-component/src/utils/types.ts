import {
  ChunkGroup,
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

export type GroupChunk = {
  chunks: ChunkWithHighlights[];
  group: ChunkGroup;
};

export type ChunkWithHighlights = { chunk: Chunk; highlights: string[] };

export type SearchResults = {
  chunks: ChunkWithHighlights[];
  requestID: string;
};

export type GroupSearchResults = {
  groups: GroupChunk[];
  requestID: string;
};

export function isChunkWithHighlights(
  result: ChunkWithHighlights | GroupChunk[],
): result is ChunkWithHighlights {
  return !Array.isArray(result);
}

export function isGroupChunk(
  result: ChunkWithHighlights | GroupChunk,
): result is GroupChunk {
  return (result as GroupChunk).group !== undefined;
}

export type Props = {
  datasetId: string;
  apiKey: string;
  onResultClick?: (chunk: Chunk, requestID: string) => void;
  theme?: "light" | "dark";
  searchOptions?: Omit<
    SearchChunksReqPayload,
    "query" | "highlight_options"
  > & {
    use_autocomplete?: boolean;
  };
  placeholder?: string;
};
