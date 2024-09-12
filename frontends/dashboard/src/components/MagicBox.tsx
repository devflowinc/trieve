import { CreateQueryResult } from "@tanstack/solid-query";
import { cva, VariantProps } from "cva";
import { cn } from "shared/utils";
import { createMemo, JSX } from "solid-js";
import { Show } from "solid-js";

interface MagicBoxProps<D extends CreateQueryResult>
  extends VariantProps<typeof container> {
  class?: string;
  fallback?: JSX.Element;
  id?: string;
  heightKey?: string;
  query: D;
  children: (data: NonNullable<D["data"]>) => JSX.Element;
}

const container = cva([], {
  variants: {
    styled: {
      true: "bg-white border border-neutral-300 p-4 shadow-sm",
      false: "",
    },
  },
  defaultVariants: {
    styled: true,
  },
});

export const MagicBox = <D extends CreateQueryResult>(
  props: MagicBoxProps<D>,
) => {
  const children = createMemo(() => {
    return props.children(props.query.data as NonNullable<D["data"]>);
  });

  const skeletonHeight = createMemo(() => {
    if (props.heightKey) {
      if (props.query.status === "success") {
        console.log("saving height");
        // save height of div to local storage
        const height = document.getElementById(`skeleton-${props.heightKey}`)
          ?.clientHeight;
        if (height) {
          console.log("saving height", height);
          localStorage.setItem(
            `skeleton-${props.heightKey}`,
            height.toString(),
          );
        }
      } else {
        // get height from local storage
        const height = localStorage.getItem(`skeleton-${props.heightKey}`);
        if (height) {
          console.log("restoring height", height);
          return `${height}px`;
        } else {
          return "auto";
        }
      }
    } else {
      return "auto";
    }
  });

  return (
    <div
      style={{ height: skeletonHeight() }}
      id={`skeleton-${props.heightKey}`}
      class={cn(
        container({ ...props, class: props.class }),
        props.query.isLoading && "shimmer",
      )}
    >
      <Show fallback={props.fallback} when={props.query.data}>
        {children()}
      </Show>
    </div>
  );
};
