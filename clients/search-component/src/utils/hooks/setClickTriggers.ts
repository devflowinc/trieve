import { startTransition } from "react";
import { ModalProps, SearchModes } from "./modal-context";

export const setClickTriggers = (
  setOpen: (open: boolean) => void,
  setMode: React.Dispatch<React.SetStateAction<SearchModes>>,
  props: ModalProps,
) => {
  const removeAllClickListeners = (selector: string): Element | null => {
    const element: Element | null = document.querySelector(selector);
    if (!element) return null;
    // Vue click attributes
    element.removeAttribute("@click.prevent");
    element.removeAttribute("@click");
    // @ts-expect-error Property 'href' does not exist on type 'Element'. [2339]
    element.href = "#";

  const detailsElement = element.querySelector('details');
  
  if (detailsElement) {
    // Find and remove the summary element
    const summaryElement = detailsElement.querySelector('summary');
    
    if (summaryElement) {
      //keep everything under the summary element
      summaryElement.querySelector(".header__icon-close")?.remove();
      const newDetailsChild = summaryElement.cloneNode(true);
      detailsElement.replaceChildren(newDetailsChild);
    }
  }

    const newElement = element.cloneNode(true);
    element?.parentNode?.replaceChild(newElement, element);
    return newElement as unknown as Element;
  };

  props.buttonTriggers?.forEach((trigger) => {
    let element: Element | null = document.querySelector(trigger.selector);
    if (trigger.removeListeners ?? true) {
      element = removeAllClickListeners(trigger.selector);
    }

    if (element) {
      element.addEventListener("click", () => {
        startTransition(() => {
          setMode(trigger.mode);
          setOpen(true);
        });
      });
    }
  });
};
