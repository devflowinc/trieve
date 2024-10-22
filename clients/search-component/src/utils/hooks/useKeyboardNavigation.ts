import { useEffect, useMemo } from "react";
import { useModalState } from "./modal-context";

export const useKeyboardNavigation = () => {
  const { setOpen, props, open, inputRef } = useModalState();

  const keyCombo = props.openKeyCombination || [{ ctrl: true }, { key: "k" }];

  const checkForInteractions = useMemo(() => {
    return (e: KeyboardEvent) => {
      if (!open) {
        const hasCtrl = keyCombo.find((k) => k.ctrl);
        if ((hasCtrl && (e.metaKey || e.ctrlKey)) || !hasCtrl) {
          const otherKeys = keyCombo.filter((k) => !k.ctrl);
          if (otherKeys.every((k) => e.key === k.key)) {
            e.preventDefault();
            e.stopPropagation();
            setOpen(true);
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
          console.log("focusedElement", focusedElement);

          if (id && id.startsWith("trieve-search-item-")) {
            const index = parseInt(id.split("-")[3]);
            console.log("index", index);
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
    };
  }, [open]);

  useEffect(() => {
    document.addEventListener("keydown", checkForInteractions);
    return () => {
      document.removeEventListener("keydown", checkForInteractions);
    };
  }, [checkForInteractions]);
};
