import { useEffect } from "react";
import { useModalState } from "./modal-context";

export const useKeyboardNavigation = () => {
  const { results, setOpen, inputRef, props, open } = useModalState();

  const keyCombo = props.openKeyCombination || [{ ctrl: true }, { key: "k" }];

  const checkForInteractions = (e: KeyboardEvent) => {
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

    // if (e.code === "ArrowDown" && inputRef.current === document.activeElement) {
    //   document.getElementById(`trieve-search-item-0`)?.focus();
    // }
  };

  useEffect(() => {
    document.addEventListener("keydown", checkForInteractions);
    return () => {
      document.removeEventListener("keydown", checkForInteractions);
    };
  }, []);

  const onUpOrDownClicked = (index: number, code: string) => {
    if (code === "ArrowDown") {
      if (index < results.length - 1) {
        // document.getElementById(`trieve-search-item-${index + 1}`)?.focus();
      } else {
        // document.getElementById(`trieve-search-item-0`)?.focus();
      }
    }

    if (code === "ArrowUp") {
      if (index > 0) {
        // document.getElementById(`trieve-search-item-${index - 1}`)?.focus();
      } else {
        // inputRef.current?.focus();
      }
    }
  };
  return { onUpOrDownClicked };
};
