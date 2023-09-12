import { Show, createSignal } from "solid-js";
import type { JSX } from "solid-js/jsx-runtime";

export interface TooltipProps {
  body: JSX.Element;
  tooltipText: string;
}

export const Tooltip = (props: TooltipProps) => {
  const [show, setShow] = createSignal(false);

  return (
    <div class="relative">
      <div
        class=" hover:cursor-help"
        onMouseEnter={() => setShow(true)}
        onMouseLeave={() => setShow(false)}
      >
        {props.body}
      </div>
      <Show when={show()}>
        <div class="absolute z-10 inline-block w-[100px] -translate-x-[45%] translate-y-3 rounded bg-white p-2 text-center shadow-lg dark:bg-black">
          {props.tooltipText}
        </div>
        <div class="caret dark:bg-shark-700 absolute h-4 w-4 translate-x-[2px] translate-y-2 rotate-45 transform bg-white" />
      </Show>
    </div>
  );
};
