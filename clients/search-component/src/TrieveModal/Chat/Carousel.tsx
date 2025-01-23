import React, {
  useState,
  useEffect,
  useRef,
  Children,
  useCallback,
} from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ArrowLeftIcon, ArrowRightIcon } from "../icons";

export const Carousel = ({ children }: { children: React.ReactNode }) => {
  const [itemsPerPage, setItemsPerPage] = useState(1);
  const [showLeftArrow, setShowLeftArrow] = useState(false);
  const [showRightArrow, setShowRightArrow] = useState(true);

  const scrollRef = useRef<HTMLUListElement>(null);
  const rootRef = useRef<HTMLDivElement>(null);

  const calcItemsPerPage = useCallback(() => {
    if (rootRef.current) {
      const width = rootRef.current.getBoundingClientRect().width;
      const itemsFit = Math.max(1, Math.floor(width / (12 * 16)));
      setItemsPerPage(itemsFit);
    }
  }, []);

  useEffect(() => {
    const resizeObserver = new ResizeObserver(() => {
      calcItemsPerPage();
    });

    if (rootRef.current) {
      resizeObserver.observe(rootRef.current);
      calcItemsPerPage();
    }

    return () => {
      resizeObserver.disconnect();
    };
  }, [calcItemsPerPage]);

  const productItems = Children.toArray(children);

  const handleArrowClick = (direction: "left" | "right") => {
    if (scrollRef.current) {
      const scrollAmount = scrollRef.current.offsetWidth / itemsPerPage;
      scrollRef.current.scrollBy({
        left: direction === "left" ? -scrollAmount : scrollAmount,
        behavior: "smooth",
      });
    }
  };

  const checkScrollPosition = useCallback(() => {
    if (scrollRef.current) {
      const { scrollLeft, scrollWidth, clientWidth } = scrollRef.current;
      setShowLeftArrow(scrollLeft > 0);
      setShowRightArrow(Math.ceil(scrollLeft + clientWidth) < scrollWidth);
    }
  }, []);

  useEffect(() => {
    const scrollElement = scrollRef.current;
    if (scrollElement) {
      scrollElement.addEventListener("scroll", checkScrollPosition);
      checkScrollPosition();
    }
    return () => {
      if (scrollElement) {
        scrollElement.removeEventListener("scroll", checkScrollPosition);
      }
    };
  }, [checkScrollPosition]);

  useEffect(() => {
    checkScrollPosition();
  }, [children, itemsPerPage, checkScrollPosition]);

  return (
    <div
      className="carousel-root tv-relative tv-w-full tv-max-w-full tv-overflow-hidden"
      ref={rootRef}
    >
      <div
        className="tv-absolute tv-left-0 tv-top-0 tv-bottom-0 tv-w-12 tv-pointer-events-none tv-z-10"
        style={{
          background:
            "linear-gradient(to right, rgb(255, 255, 255, 0.8) 0%, rgba(255, 255, 255, 0) 100%)",
          opacity: showLeftArrow ? 0.4 : 0,
          transition: "opacity 0.2s ease",
        }}
      />

      <AnimatePresence>
        {showLeftArrow && (
          <motion.button
            initial={{ opacity: 0, x: -10 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -10 }}
            transition={{ duration: 0.2 }}
            className="carousel-arrow tv-absolute tv-top-[43%] tv-left-2 tv-text-gray-100 tv-bg-gray-900/65 tv-rounded-full tv-px-4 tv-py-3 tv-cursor-pointer tv-z-[999]"
            onClick={() => handleArrowClick("left")}
          >
            <ArrowLeftIcon className="tv-w-2 tv-h-4" />
          </motion.button>
        )}
      </AnimatePresence>
      <ul
        className="carousel-scroll tv-flex tv-gap-4 tv-m-0 tv-overflow-y-hidden tv-scroll-smooth tv-scrollbar-none"
        ref={scrollRef}
        style={{
          overflowX: "scroll",
        }}
      >
        {productItems.map((child, index) => (
          <motion.li
            className={`carousel-item tv-flex-shrink-0 tv-list-none tv-rounded-lg tv-overflow-hidden tv-bg-white tv-shadow-sm tv-border-2`}
            key={index}
            style={{
              width: `min(calc((100% / ${itemsPerPage}) - 1rem), calc(90% - 1rem))`,
              transition: "border-color 0.2s ease",
            }}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3, delay: index * 0.1 }}
          >
            <div className="tv-flex tv-flex-col tv-items-center tv-h-full tv-w-full tv-box-border">
              {child}
            </div>
          </motion.li>
        ))}
      </ul>
      <div
        className="tv-absolute tv-right-0 tv-top-0 tv-bottom-0 tv-w-12 tv-pointer-events-none tv-z-10"
        style={{
          background:
            "linear-gradient(to left, rgb(255, 255, 255, 0.8) 0%, rgba(255, 255, 255, 0) 100%)",
          opacity: showRightArrow ? 0.4 : 0,
          transition: "opacity 0.2s ease",
        }}
      />

      <AnimatePresence>
        {showRightArrow && (
          <motion.button
            initial={{ opacity: 0, x: 10 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 10 }}
            transition={{ duration: 0.2 }}
            className="carousel-arrow tv-absolute tv-top-[43%] tv-right-2 tv-text-gray-100 tv-bg-gray-900/65 tv-rounded-full tv-px-4 tv-py-3 tv-cursor-pointer tv-z-[999]"
            onClick={() => handleArrowClick("right")}
          >
            <ArrowRightIcon className="tv-w-2 tv-h-4" />
          </motion.button>
        )}
      </AnimatePresence>
    </div>
  );
};
