import { startTransition } from "react";
import { useModalState } from "./modal-context";

export const setClickTriggers = () => {
  const { setOpen, setMode, props } = useModalState();

  props.buttonTriggers?.forEach((trigger) => {
    const element = document.querySelector(trigger.selector);
    if (element) {
      element.addEventListener("click", () => {
        startTransition(() => {
          setMode(trigger.mode);
          setOpen(true);
        })
      });
    }
  })
}
