import { startTransition, useCallback, useEffect, useRef } from "react";
import { useModalState } from "./modal-context";

export const useKeyboardNavigation = () => {
  const { setOpen, props, open, inputRef } = useModalState();

  const keyCombo = props.openKeyCombination || [{ ctrl: true }, { key: "k" }];
  const lastButtonInteraction = useRef<number>(0);

  // Check if interaction was within last 10 seconds
  const isWithinTimeWindow = () => {
    if (lastButtonInteraction.current === 0) {
      return true;
    }
    return Date.now() - lastButtonInteraction.current >= 10;
  };

  const checkForInteractions = useCallback(
    (e: KeyboardEvent) => {
      if (!isWithinTimeWindow()) {
        return;
      }
      lastButtonInteraction.current = Date.now();

      if (!open) {
        const hasCtrl = keyCombo.find((k) => k.ctrl);
        if ((hasCtrl && (e.metaKey || e.ctrlKey)) || !hasCtrl) {
          const otherKeys = keyCombo.filter((k) => !k.ctrl);
          if (otherKeys.every((k) => e.key === k.key)) {
            e.preventDefault();
            e.stopPropagation();
            startTransition(() => {
              setOpen(true);
            });
          }
        }
      }

      if (open && e.key === "Escape") {
        setOpen(false);
      } else if (open) {
        if (e.key == "ArrowDown") {
          e.preventDefault();
          e.stopPropagation();

          const focusedElement = document.activeElement as HTMLElement;
          const id = focusedElement.id;

          if (id && id.startsWith("trieve-search-item-")) {
            const index = parseInt(id.split("-")[3]);
            document.getElementById(`trieve-search-item-${index + 1}`)?.focus();
          }

          if (!id || !id.startsWith("trieve-search-item-")) {
            document.getElementById(`trieve-search-item-0`)?.focus();
          }
        } else if (e.key == "ArrowUp") {
          e.preventDefault();
          e.stopPropagation();

          const focusedElement = document.activeElement as HTMLElement;
          const id = focusedElement.id;
          if (id && id.startsWith("trieve-search-item-")) {
            const index = parseInt(id.split("-")[3]);
            if (index > 0) {
              document
                .getElementById(`trieve-search-item-${index - 1}`)
                ?.focus();
            } else {
              inputRef.current?.focus();
            }
          }
        }
      }
    },
    [open],
  );

  useEffect(() => {
    document.addEventListener("keydown", checkForInteractions);
    return () => {
      document.removeEventListener("keydown", checkForInteractions);
    };
  }, [checkForInteractions]);
};
