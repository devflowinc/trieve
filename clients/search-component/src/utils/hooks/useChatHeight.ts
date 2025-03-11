// In your useChatHeight hook
import { atom, useAtom } from "jotai";
import { useEffect } from "react";

const chatHeightAtom = atom<number>(0);
const enabledAtom = atom<boolean>(true);
const minHeightAtom = atom<number>(0);
const contentHeightAtom = atom<number>(0); // new atom to store content height

export const useChatHeight = (
  modalRef?: React.RefObject<HTMLDivElement>,
  absoluteMinimum: number = 0,
) => {
  const [minHeight, setMinHeight] = useAtom(minHeightAtom);
  const [chatHeight, setChatHeight] = useAtom(chatHeightAtom);
  const [enabled, setEnabled] = useAtom(enabledAtom);
  const [contentHeight, setContentHeight] = useAtom(contentHeightAtom); // content height state

  useEffect(() => {
    if (!modalRef || !modalRef.current) {
      return;
    }
    const ref = modalRef.current;
    const observer = new ResizeObserver((entries) => {
      setChatHeight(entries[0].contentRect.height);
    });

    const contentObserver = new MutationObserver(() => {
      if (modalRef.current) {
        setContentHeight(modalRef.current.scrollHeight);
      }
    });

    observer.observe(ref);
    contentObserver.observe(ref, {
      childList: true,
      subtree: true,
      characterData: true,
    }); // observe content changes

    return () => {
      observer.disconnect();
      contentObserver.disconnect();
    };
  }, [modalRef]);

  useEffect(() => {
    if (chatHeight > minHeight && enabled) {
      setMinHeight(chatHeight);
    }
  }, [chatHeight, minHeight, enabled]);

  const resetHeight = () => {
    setMinHeight(absoluteMinimum);
    setEnabled(false);
    setTimeout(() => {
      setEnabled(true);
    }, 200);
  };

  const addHeight = (height: number) => {
    setMinHeight((prev) => prev + height);
  };

  return { minHeight, resetHeight, addHeight, contentHeight }; // return contentHeight
};
