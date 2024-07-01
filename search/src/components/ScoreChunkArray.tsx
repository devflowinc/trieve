import type { Setter } from "solid-js";
import { Show, createSignal, onMount } from "solid-js";
import {
  indirectHasOwnProperty,
  type ChunkGroupDTO,
  type ChunkMetadata,
} from "../../utils/apiTypes";
import type { ScoreChunkProps } from "./ScoreChunk";
import { FiChevronLeft, FiChevronRight } from "solid-icons/fi";
import ScoreChunk from "./ScoreChunk";
import { defaultClientEnvsConfiguration } from "../../utils/apiTypes";

export type ScoreChunkAraryProps = Omit<
  ScoreChunkProps,
  "chunk" | "counter" | "total" | "begin" | "end"
> & {
  chunks: ChunkMetadata[];
  setChunkGroups: Setter<ChunkGroupDTO[]>;
};

export const ScoreChunkArray = (props: ScoreChunkAraryProps) => {
  const [curChunk, setCurChunk] = createSignal(0);
  const [beginTime, setBeginTime] = createSignal<number | undefined>();
  const [endTime, setEndTime] = createSignal<number | undefined>();

  onMount(() => {
    const dateValue = defaultClientEnvsConfiguration.DATE_RANGE_VALUE;

    props.chunks.forEach((chunk) => {
      if (
        chunk.metadata &&
        dateValue != undefined &&
        dateValue != "" &&
        indirectHasOwnProperty(chunk.metadata, dateValue)
      ) {
        // regex to select only valid dates
        // (\d{1,4}([.\-/])\d{1,2}([.\-/])\d{1,4})
        const dateString = chunk.metadata[dateValue] as string;
        const dateRegex = /(\d{1,4}([.\-/])\d{1,2}([.\-/])\d{1,4})/;
        // extract the first match from the string
        const dateMatch = dateString.match(dateRegex)?.[0];
        if (!dateMatch) return;

        // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
        const dateObject = new Date(dateMatch);
        if (dateObject.getTime()) {
          setBeginTime((prev) =>
            Math.min(prev ?? Infinity, dateObject.getTime()),
          );
          setEndTime((prev) => Math.max(prev ?? 0, dateObject.getTime()));
        }
      }
    });
  });
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
        begin={beginTime()}
        end={endTime()}
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
