import { createMemo, createSignal } from "solid-js";

export const usePagination = () => {
  const [page, setPage] = createSignal(1);
  const [maxPageDiscovered, setMaxPageDiscovered] = createSignal<number | null>(
    null,
  );

  const nextPage = () => {
    setPage((prev) => prev + 1);
  };

  const prevPage = () => {
    if (page() > 1) {
      setPage((prev) => prev - 1);
    }
  };

  const canGoNext = createMemo(() => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const maxPage = maxPageDiscovered();
    return maxPage === null || page() < maxPage;
  });

  const resetMaxPageDiscovered = () => {
    setMaxPageDiscovered(null);
  };

  return {
    page,
    nextPage,
    prevPage,
    canGoNext,
    setMaxPageDiscovered,
    resetMaxPageDiscovered,
    maxPageDiscovered,
  };
};
