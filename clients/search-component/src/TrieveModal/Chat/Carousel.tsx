import React, {
  useState,
  useEffect,
  useRef,
  CSSProperties,
  Children,
  useCallback,
} from "react";

const styles = {
  nextPrevButton: {
    fontSize: "16px",
    cursor: "pointer",
    backgroundColor: "transparent",
    border: "none",
    padding: "10px",
    margin: "0 5px",
  },
  nextPrevButtonDisabled: {
    opacity: 0.3,
    cursor: "not-allowed",
  },
  paginationButton: {
    margin: "10px",
  },
  paginationButtonActive: {
    opacity: 0.3,
  },
} satisfies Record<string, CSSProperties>;

export const Carousel = ({ children }: { children: React.ReactNode }) => {
  const [currentPage, setCurrentPage] = useState(0);
  const [itemsPerPage, setItemsPerPage] = useState(1);

  const scrollRef = useRef<HTMLUListElement>(null);

  const calcItemsPerPage = useCallback(() => {
    const containerWidth = scrollRef.current?.offsetWidth ?? window.innerWidth;

    const itemsFit = Math.max(1, Math.floor(containerWidth / (12 * 16)));

    setItemsPerPage(itemsFit);
  }, []);

  function onVisible(element: HTMLElement, callback: () => void) {
    new IntersectionObserver((entries, observer) => {
      entries.forEach((entry) => {
        if (entry.intersectionRatio > 0) {
          callback();
          observer.disconnect();
        }
      });
    }).observe(element);
  }

  useEffect(() => {
    calcItemsPerPage();

    window.addEventListener("resize", calcItemsPerPage);
    onVisible(document.querySelector(".carousel-root")!, calcItemsPerPage);
    return () => {
      window.removeEventListener("resize", calcItemsPerPage);
    };
  }, []);

  const productItems = Children.toArray(children);
  const numItems = productItems.length;
  const numPages = Math.ceil(numItems / itemsPerPage);

  const placeholderItems = Array(itemsPerPage - (numItems % itemsPerPage)).fill(
    null,
  );

  const allProductsCarousel =
    numItems % itemsPerPage != 0
      ? [...productItems, ...placeholderItems]
      : productItems;

  useEffect(() => {
    if (scrollRef.current) {
      const itemWidth = scrollRef.current.offsetWidth / itemsPerPage;
      const scrollLeft = currentPage * itemWidth * itemsPerPage;

      scrollRef.current.scrollTo({
        left: scrollLeft,
        behavior: "smooth",
      });
    }
  }, [children, currentPage, itemsPerPage]);

  const prev = () => setCurrentPage((prev) => Math.max(prev - 1, 0));
  const next = () => setCurrentPage((prev) => Math.min(prev + 1, numPages - 1));

  const goToPage = (pageIndex: number) => setCurrentPage(pageIndex);

  return (
    <div className="carousel-root">
      <ul className="carousel-scroll" ref={scrollRef}>
        {allProductsCarousel.map((child, index) => (
          <li
            className={
              child
                ? "carousel-item carousel-item-visibile"
                : "carousel-item carousel-item-hidden"
            }
            style={{
              width: `calc(100% / ${itemsPerPage})`,
            }}
            key={index}>
            {child}
          </li>
        ))}
      </ul>
      {numPages > 1 && (
        <div className="carousel-controls">
          <button
            style={{
              ...styles.nextPrevButton,
              ...(currentPage == 0 ? styles.nextPrevButtonDisabled : {}),
            }}
            onClick={() => prev()}
            disabled={currentPage == 0}>
            {String.fromCharCode(8592)}
          </button>
          {[...Array(numPages)].map((_, i) => (
            <button
              key={i}
              style={{
                ...styles.paginationButton,
                ...(currentPage == i ? styles.paginationButtonActive : {}),
              }}
              onClick={() => goToPage(i)}>
              {i + 1}
            </button>
          ))}
          <button
            style={{
              ...styles.nextPrevButton,
              ...(currentPage === numPages - 1
                ? styles.nextPrevButtonDisabled
                : {}),
            }}
            onClick={() => next()}
            disabled={currentPage === numPages - 1}>
            {String.fromCharCode(8594)}
          </button>
        </div>
      )}
    </div>
  );
};
