import React, {
  useState,
  useEffect,
  useRef,
  CSSProperties,
  Children,
} from "react";

const styles = {
  root: {
    width: "100%",
    overflow: "hidden",
  },
  scroll: {
    display: "flex",
    overflowX: "hidden",
    scrollSnapType: "x mandatory",
    scrollBehavior: "smooth",
    width: "100%",
    listStyle: "none",
    padding: 0,
    margin: 0,
  },
  item: {
    flex: "0 0 auto",
    padding: "1rem",
    scrollSnapAlign: "start",
    boxSizing: "border-box",
  },
  itemSnapPoint: {
    scrollSnapAlign: "start",
  },
  controls: {
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    marginTop: "5px",
  },
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

  useEffect(() => {
    const calcItemsPerPage = () => {
      const containerWidth =
        scrollRef.current?.offsetWidth ?? window.innerWidth;

      const itemsFit = Math.max(1, Math.floor(containerWidth / (12 * 16)));

      setItemsPerPage(itemsFit);
    };

    calcItemsPerPage();

    window.addEventListener("resize", calcItemsPerPage);
    return () => window.removeEventListener("resize", calcItemsPerPage);
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
    <div style={styles.root}>
      <ul style={styles.scroll} ref={scrollRef}>
        {allProductsCarousel.map((child, index) => (
          <li
            style={{
              ...styles.item,
              width: `calc(100% / ${itemsPerPage})`,
              visibility: child ? "visible" : "hidden",
            }}
            key={index}
          >
            {child}
          </li>
        ))}
      </ul>
      {numPages > 1 && (
        <div style={styles.controls}>
          <button
            style={{
              ...styles.nextPrevButton,
              ...(currentPage == 0 ? styles.nextPrevButtonDisabled : {}),
            }}
            onClick={() => prev()}
            disabled={currentPage == 0}
          >
            {String.fromCharCode(8592)}
          </button>
          {[...Array(numPages)].map((_, i) => (
            <button
              key={i}
              style={{
                ...styles.paginationButton,
                ...(currentPage == i ? styles.paginationButtonActive : {}),
              }}
              onClick={() => goToPage(i)}
            >
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
            disabled={currentPage === numPages - 1}
          >
            {String.fromCharCode(8594)}
          </button>
        </div>
      )}
    </div>
  );
};
