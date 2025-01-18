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
  transcribedQuery?: string;
};

export type GroupSearchResults = {
  groups: GroupChunk[];
  requestID: string;
  transcribedQuery?: string;
};

export function isChunkWithHighlights(
  result: ChunkWithHighlights | GroupChunk[],
): result is ChunkWithHighlights {
  return !Array.isArray(result);
}

export type PdfChunk = {
  chunk: Chunk & {
    metadata: {
      file_name: string;
      page_num: number;
      file_id: string;
    };
  };
  highlights: string[];
};

export function isPdfChunk(result: ChunkWithHighlights): result is PdfChunk {
  return (
    (result as PdfChunk).chunk.metadata.file_name !== undefined &&
    (result as PdfChunk).chunk.metadata.page_num !== undefined
  );
}

export function isSimplePdfChunk(result: Chunk): result is PdfChunk["chunk"] {
  return (
    (result as PdfChunk["chunk"]).metadata.file_name !== undefined &&
    (result as PdfChunk["chunk"]).metadata.page_num !== undefined
  );
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
