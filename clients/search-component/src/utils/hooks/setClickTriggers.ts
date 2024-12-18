import { startTransition } from "react";
import { useModalState } from "./modal-context";

export const setClickTriggers = () => {
  const { setOpen, setMode, props } = useModalState();

  function removeAllClickListeners(selector: string) {
    const element: Element | null = document.querySelector(selector);
    if (!element) return;
    // Vue click attributes
    element.removeAttribute("@click.prevent");
    element.removeAttribute("@click");

    const newElement = element.cloneNode(true);
    element?.parentNode?.replaceChild(newElement, element);
  }


  props.buttonTriggers?.forEach((trigger) => {
    const element = document.querySelector(trigger.selector);
    if (element) {
      if (trigger.removeListeners ?? true) {
        removeAllClickListeners(trigger.selector);
      }
      element.addEventListener("click", () => {
        startTransition(() => {
          setMode(trigger.mode);
          setOpen(true);
        })
      });
    }
  })
}
