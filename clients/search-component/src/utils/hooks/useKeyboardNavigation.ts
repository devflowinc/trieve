import { useEffect } from "react";
import { useModalState } from "./modal-context";

export const useKeyboardNavigation = () => {
  const { results, setOpen, inputRef } = useModalState();
  const checkForInteractions = (e: KeyboardEvent) => {
    if (e.code === "KeyK" && !open && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      e.stopPropagation();
      setOpen(true);
    }

    if (e.code === "ArrowDown" && inputRef.current === document.activeElement) {
      document.getElementById(`trieve-search-item-0`)?.focus();
    }
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
        document.getElementById(`trieve-search-item-${index + 1}`)?.focus();
      } else {
        document.getElementById(`trieve-search-item-0`)?.focus();
      }
    }

    if (code === "ArrowUp") {
      if (index > 0) {
        document.getElementById(`trieve-search-item-${index - 1}`)?.focus();
      } else {
        inputRef.current?.focus();
      }
    }
  };
  return { onUpOrDownClicked };
};
