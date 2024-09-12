import { CreateQueryResult } from "@tanstack/solid-query";
import { cva, VariantProps } from "cva";
import { cn } from "shared/utils";
import {
  createEffect,
  createMemo,
  createSignal,
  JSX,
  onCleanup,
  onMount,
  Suspense,
} from "solid-js";
import { Show } from "solid-js";

interface MagicBoxProps<D extends CreateQueryResult>
  extends VariantProps<typeof container> {
  class?: string;
  fallback?: JSX.Element;
  id?: string;
  skeletonKey?: string;
  skeletonHeight?: string;
  query: D;
  children: (data: NonNullable<D["data"]>) => JSX.Element;
}

const container = cva([], {
  variants: {
    unstyled: {
      false: "bg-white rounded-md border border-neutral-300 p-4 shadow-sm",
      true: "",
    },
  },
  defaultVariants: {
    unstyled: false,
  },
});

export const MagicBox = <D extends CreateQueryResult>(
  props: MagicBoxProps<D>,
) => {
  const children = createMemo(() => {
    return props.children(props.query.data as NonNullable<D["data"]>);
  });

  const skeletonHeight = createMemo(() => {
    if (props.skeletonKey) {
      if (props.query.status === "success") {
        // save height of div to local storage
        const height = document.getElementById(`skeleton-${props.skeletonKey}`)
          ?.clientHeight;
        if (height) {
          localStorage.setItem(
            `skeleton-${props.skeletonKey}`,
            height.toString(),
          );
        }
      } else {
        // get height from local storage
        const height = localStorage.getItem(`skeleton-${props.skeletonKey}`);
        if (height) {
          return `${height}px`;
        } else {
          return "auto";
        }
      }
    } else {
      if (props.query.isLoading) {
        if (props.skeletonHeight) {
          return `${props.skeletonHeight}px`;
        } else {
          return "auto";
        }
      }
      return "auto";
    }
  });

  return (
    <div
      style={{ height: skeletonHeight() }}
      id={`skeleton-${props.skeletonKey}`}
      class={cn(
        container({ ...props, class: props.class }),
        props.query.isLoading &&
          (props.unstyled ? "unstyled-shimmer" : "shimmer"),
      )}
    >
      <Show fallback={props.fallback} when={props.query.data}>
        {children()}
      </Show>
    </div>
  );
};

interface MagicSuspenseProps extends VariantProps<typeof container> {
  class?: string;
  fallback?: JSX.Element;
  id?: string;
  skeletonKey?: string;
  skeletonHeight?: string;
  children: JSX.Element;
}

export const MagicSuspense = (props: MagicSuspenseProps) => {
  const [isLoaded, setIsLoaded] = createSignal(false);
  const shimmerID = `shimmer-${
    // eslint-disable-next-line solid/reactivity
    props.skeletonKey || Math.random().toString(36).substr(2, 9)
  }`;

  const skeletonHeight = createMemo(() => {
    if (isLoaded()) return "auto";
    if (props.skeletonKey) {
      const height = localStorage.getItem(`skeleton-${props.skeletonKey}`);
      return height ? `${height}px` : props.skeletonHeight || "auto";
    }
    return props.skeletonHeight || "auto";
  });

  onMount(() => {
    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        if (mutation.type === "childList") {
          const removedNodes = Array.from(mutation.removedNodes);
          if (removedNodes.some((node) => (node as Element).id === shimmerID)) {
            setIsLoaded(true);
            observer.disconnect();
            break;
          }
        }
      }
    });

    const container = document.getElementById(`skeleton-${props.skeletonKey}`);
    if (container) {
      observer.observe(container, { childList: true, subtree: true });
    }

    onCleanup(() => observer.disconnect());
  });

  createEffect(() => {
    if (isLoaded() && props.skeletonKey) {
      const element = document.getElementById(`skeleton-${props.skeletonKey}`);
      if (element) {
        const height = element.clientHeight;
        localStorage.setItem(
          `skeleton-${props.skeletonKey}`,
          height.toString(),
        );
      }
    }
  });

  return (
    <div
      style={{ height: skeletonHeight() }}
      id={`skeleton-${props.skeletonKey}`}
      class={cn("relative", container({ ...props, class: props.class }))}
    >
      <Suspense
        fallback={
          props.fallback || (
            <div
              id={shimmerID}
              class={cn(
                "absolute bottom-0 left-0 right-0 top-0",
                props.unstyled ? "unstyled-shimmer" : "shimmer",
              )}
            />
          )
        }
      >
        {props.children}
      </Suspense>
    </div>
  );
};
