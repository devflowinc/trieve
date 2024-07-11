import { Show, createMemo, createSignal } from "solid-js";
import type { JSX } from "solid-js/jsx-runtime";

export type TooltipDirection = "top" | "bottom" | "left" | "right";

export interface TooltipProps {
  body: JSX.Element;
  tooltipText: string;
  direction?: TooltipDirection;
}

export const Tooltip = (props: TooltipProps) => {
  const [show, setShow] = createSignal(false);
  const direction = createMemo(() => props.direction || "bottom");

  return (
    <div class="relative inline-block">
      <div
        class="flex items-center hover:cursor-help"
        onMouseEnter={() => setShow(true)}
        onMouseLeave={() => setShow(false)}
      >
        {props.body}
      </div>
      <Show when={show()}>
        <div
          classList={{
            "absolute z-10 inline-block w-[300px] rounded bg-white p-2 text-center shadow-lg dark:bg-black":
              true,
            "bottom-full left-1/2 -translate-x-1/2 translate-y-3":
              direction() === "top",
            "right-full top-1/2 -translate-y-1/2 -translate-x-3":
              direction() === "left",
            "left-full top-1/2 -translate-y-1/2 translate-x-3":
              direction() === "right",
            "top-full left-1/2 -translate-x-1/2 -translate-y-3":
              direction() === "bottom",
          }}
        >
          {props.tooltipText}
        </div>
        <div
          classList={{
            "caret absolute h-4 w-4 rotate-45 transform": true,
            "bottom-0 left-1/2 -translate-x-1/2 translate-y-1/2":
              direction() === "top",
            "right-0 top-1/2 -translate-y-1/2 translate-x-1/2":
              direction() === "left",
            "left-0 top-1/2 -translate-y-1/2 -translate-x-1/2":
              direction() === "right",
            "top-0 left-1/2 -translate-x-1/2 -translate-y-1/2":
              direction() === "bottom",
          }}
        />
      </Show>
    </div>
  );
};
