import { ChunkMetadata } from "../../../ts-sdk/dist/types.gen";

export type Chunk = ChunkMetadata & {
  highlight?: string | undefined | null;
};
