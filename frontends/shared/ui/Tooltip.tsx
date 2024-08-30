import { Show, createMemo, createSignal } from "solid-js";
import type { JSX } from "solid-js/jsx-runtime";

export type TooltipDirection = "top" | "bottom" | "left" | "right";

export interface TooltipProps {
  body?: JSX.Element;
  children?: JSX.Element;
  tooltipText: string;
  direction?: TooltipDirection;
  tooltipClass?: string;
  unsetWidth?: boolean;
}

export const Tooltip = (props: TooltipProps) => {
  const [show, setShow] = createSignal(false);
  const direction = createMemo(() => props.direction || "bottom");

  return (
    <div class="relative">
      <div
        class="cursor-help flex items-center"
        onMouseEnter={() => {
          setShow(true);
        }}
        onMouseLeave={() => {
          setShow(false);
        }}
      >
        {props.children ? props.children : props.body}
      </div>
      <Show when={show()}>
        <div
          class={props.tooltipClass}
          classList={{
            "absolute z-10 inline-block rounded bg-[#fffee6] p-2 text-center shadow-lg dark:bg-[#fffee6] text-black text-wrap":
              true,
            "w-[300px]": !props.unsetWidth,
            "bottom-full left-1/2 -translate-x-1/2 translate-y-3":
              direction() === "top",
            "right-full top-1/2 -translate-y-1/2 -translate-x-3":
              direction() === "left",
            "left-full top-1/2 -translate-y-1/2 translate-x-3":
              direction() === "right",
            "top-full left-1/2 -translate-x-1/2 translate-y-3":
              direction() === "bottom",
          }}
        >
          {props.tooltipText}
        </div>
      </Show>
    </div>
  );
};
