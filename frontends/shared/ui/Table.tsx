import { cva, VariantProps } from "cva";
import { For, JSX, Show } from "solid-js";

type TRenderFunction<D> = (item: D) => JSX.Element;

interface TableProps<D> extends VariantProps<typeof table> {
  data: D[];
  children: TRenderFunction<D>;
  // Will show if data is an empty array
  fallback?: JSX.Element;
  headers?: string[];
  class?: string;
}

const table = cva(["w-full"], {
  variants: {
    spacing: {},
    debug: {
      true: "debug",
      false: "undefined",
    },
  },
});

export const td = cva([], {
  variants: {
    spacing: {
      md: ["p-1"],
    },
    border: {
      none: undefined,
      subtle: ["border border-neutral-300"],
      strong: ["border border-neutral-700"],
    },
  },
  defaultVariants: {
    border: "none",
    spacing: "md",
  },
});

export const Table = <D,>(props: TableProps<D>) => {
  return (
    <Show when={props.data.length != 0} fallback={props.fallback}>
      <table class={table({ ...props, class: props.class })}>
        <Show when={props.headers}>
          {(headers) => (
            <thead>
              <tr>
                <For each={headers()}>
                  {(header) => (
                    <th class="text-left font-semibold">{header}</th>
                  )}
                </For>
              </tr>
            </thead>
          )}
        </Show>
        <tbody>
          <For each={props.data}>{props.children}</For>
        </tbody>
      </table>
    </Show>
  );
};

export const Tr = (props: { children: JSX.Element }) => {
  return <tr>{props.children}</tr>;
};

interface TdProps extends VariantProps<typeof td> {
  children?: JSX.Element;
  class?: string;
}

export const Td = (props: TdProps) => {
  return <td class={td({ ...props, class: props.class })}>{props.children}</td>;
};
