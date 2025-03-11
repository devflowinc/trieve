import { atom, useAtom } from "jotai";
import { useEffect } from "react";

const chatHeightAtom = atom<number>(0);
const enabledAtom = atom<boolean>(true);
const minHeightAtom = atom<number>(0);

export const useChatHeight = (
  modalRef?: React.RefObject<HTMLDivElement>,
  absoluteMinimum: number = 0,
) => {
  const [minHeight, setMinHeight] = useAtom(minHeightAtom);
  const [chatHeight, setChatHeight] = useAtom(chatHeightAtom);
  const [enabled, setEnabled] = useAtom(enabledAtom);

  useEffect(() => {
    if (!modalRef) {
      return;
    }
    const ref = modalRef.current;
    if (ref) {
      const observer = new ResizeObserver((entries) => {
        setChatHeight(entries[0].contentRect.height);
      });
      observer.observe(ref);
      return () => {
        observer.disconnect();
      };
    }
  }, [modalRef]);

  useEffect(() => {
    if (chatHeight > minHeight && enabled) {
      setMinHeight(chatHeight);
    }
  }, [chatHeight]);

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

  return { minHeight, resetHeight, addHeight };
};
