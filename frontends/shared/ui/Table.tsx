import { For, JSX, Show } from "solid-js";

type TRenderFunction<D> = (item: D) => JSX.Element;

interface TableProps<D> {
  class?: string;
  data: D[];
  children: TRenderFunction<D>;
  fallback?: JSX.Element;
  headers?: string[];
}

export const Table = <D,>(props: TableProps<D>) => {
  return (
    <Show when={props.data.length != 0} fallback={props.fallback}>
      <table class="w-full">
        <thead>
          <tr>
            <th class="text-left font-semibold">Message</th>
            <th class="text-right font-semibold">RAG Type</th>
          </tr>
        </thead>
        <tbody>
          <For each={props.data}>{props.children}</For>
        </tbody>
      </table>
    </Show>
  );
};
