import { createMemo, createSignal, For, onMount } from "solid-js";
import { ServerTiming } from "./ResultsPage";

interface ServerTimingsProps {
  timings: ServerTiming[];
}

export const ServerTimings = (props: ServerTimingsProps) => {
  const [timingsBox, setTimingsBox] = createSignal<HTMLDivElement>();
  const [fullContainer, setFullContainer] = createSignal<HTMLDivElement>();
  const [availableWidth, setAvailableWidth] = createSignal(0);
  const [fullContainerWidth, setFullContainerWidth] = createSignal(0);

  const totalTime = createMemo(() => {
    return props.timings.reduce((acc, t) => acc + t.duration, 0);
  });

  onMount(() => {
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      setAvailableWidth(entry.contentRect.width);
    });
    if (timingsBox() !== undefined) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      observer.observe(timingsBox()!);
    }

    const fullContainerObserver = new ResizeObserver((entries) => {
      const entry = entries[0];
      setFullContainerWidth(entry.contentRect.width);
    });
    if (fullContainer() !== undefined) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      fullContainerObserver.observe(fullContainer()!);
    }

    return () => {
      observer.disconnect();
      fullContainerObserver.disconnect();
    };
  });

  const labels = createMemo(() => props.timings.map((t) => t.name));

  const banners = createMemo(() => {
    if (availableWidth() === 0) return [];
    let usedWidth = 0;
    return props.timings.map((timing, idx) => {
      const width = (timing.duration / totalTime()) * availableWidth();
      const banner = (
        <div
          class="grid w-full place-items-center bg-fuchsia-300 dark:bg-fuchsia-700"
          style={{
            width: `${width}px`,
            "min-width": `${width}px`,
            "max-width": `${width}px`,
            height: "24px",
            position: "absolute",
            left: `${usedWidth}px`,
            top: `${idx * 24}px`,
          }}
        >
          <div class="h-full w-full pt-[4px] text-center align-middle text-xs">
            {timing.duration}ms
          </div>
        </div>
      );
      usedWidth += width;
      return banner;
    });
  });

  return (
    <div ref={setFullContainer} class="py-2">
      <div class="text-md pl-2 font-semibold">Total Time: {totalTime()}ms</div>
      <div class="flex">
        <div class="shrink px-2 pr-4">
          <For each={labels()}>
            {(name) => (
              <>
                <div
                  style={{
                    "min-width": `${fullContainerWidth() - 18}px`,
                  }}
                  class="absolute border border-neutral-300 dark:border-neutral-800/80"
                />
                <div class="pl-2">{name.replaceAll("_", " ")}</div>
              </>
            )}
          </For>
          <div
            style={{
              "min-width": `${fullContainerWidth() - 18}px`,
            }}
            class="absolute border border-neutral-300 dark:border-neutral-800/80"
          />
        </div>
        <div class="mr-8 grow">
          <div
            id="timingsBox"
            ref={setTimingsBox}
            class="relative grow border-l border-l-neutral-300 dark:border-l-neutral-800/80"
            style={{ height: `${props.timings.length * 24}px` }}
          >
            <For each={banners()}>{(component) => component}</For>
          </div>
        </div>
      </div>
    </div>
  );
};
