import { createSignal } from "solid-js";

export const [messageSizing, setMessageSizing] = createSignal([0.65, 0.35]);

type HandleHoverState = "none" | "hover" | "hold";
export const [handleHover, setHandleHover] =
  createSignal<HandleHoverState>("none");
