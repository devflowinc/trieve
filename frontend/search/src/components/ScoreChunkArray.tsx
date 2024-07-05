import type { Setter } from "solid-js";
import { Show, createSignal } from "solid-js";
import { type ChunkGroupDTO, type ChunkMetadata } from "../utils/apiTypes";
import type { ScoreChunkProps } from "./ScoreChunk";
import { FiChevronLeft, FiChevronRight } from "solid-icons/fi";
import ScoreChunk from "./ScoreChunk";

export type ScoreChunkAraryProps = Omit<
  ScoreChunkProps,
  "chunk" | "counter" | "total" | "begin" | "end"
> & {
  chunks: ChunkMetadata[];
  setChunkGroups: Setter<ChunkGroupDTO[]>;
};

export const ScoreChunkArray = (props: ScoreChunkAraryProps) => {
  const [curChunk, setCurChunk] = createSignal(0);

  return (
    <div class="mx-auto flex w-full max-w-[calc(100vw-32px)] items-center">
      <div class="w-[16px] min-[360px]:w-[32px]">
        <Show when={curChunk() > 0}>
          <button onClick={() => setCurChunk((prev) => prev - 1)}>
            <FiChevronLeft class="h-4 w-4 min-[360px]:h-8 min-[360px]:w-8" />
          </button>
        </Show>
        <Show when={curChunk() <= 0}>
          <FiChevronLeft class="h-4 w-4 text-transparent min-[360px]:h-8 min-[360px]:w-8" />
        </Show>
      </div>
      <ScoreChunk
        {...props}
        chunk={props.chunks[curChunk()]}
        counter={(curChunk() + 1).toString()}
        total={props.chunks.length}
        showExpand={props.showExpand}
        defaultShowMetadata={props.defaultShowMetadata}
      />
      <div class="w-[16px] min-[360px]:w-[32px]">
        <Show when={curChunk() < props.chunks.length - 1}>
          <button onClick={() => setCurChunk((prev) => prev + 1)}>
            <FiChevronRight class="h-4 w-4 min-[360px]:h-8 min-[360px]:w-8" />
          </button>
        </Show>
        <Show when={curChunk() >= props.chunks.length - 1}>
          <FiChevronRight class="h-4 w-4 text-transparent min-[360px]:h-8 min-[360px]:w-8" />
        </Show>
      </div>
    </div>
  );
};
