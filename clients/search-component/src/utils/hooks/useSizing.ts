import { cva } from "cva";
import { useModalState } from "./modal-context";

const popupClass = cva(
  [
    "tv-max-h-[40vh]",
    "tv-w-[90vw]",
    "sm:tv-max-w-[800px]",
    "tv-top-[calc(40%-30vh)] tv-left-[50%] tv-shadow-2xl tv-fixed -tv-translate-x-[50%]",
  ],
  {
    variants: {
      type: {
        ecommerce: ["tv-max-w-[90rem] tv-px-4"],
        docs: [],
        pdf: [],
      },
    },
  },
);

const inlineClass = cva(
  [
    "tv-max-h-[40vh]",
    "tv-w-[90vw]",
    "sm:tv-max-w-[800px]",
    "tv-min-w-full tv-max-w-sm !tv-w-full",
  ],
  {
    variants: {
      type: {
        ecommerce: [
          "tv-top-1 tv-w-[95vw] tv-px-4 tv-rounded-lg",
          "tv-px-0 tv-pt-0 tv-max-w-full",
        ],
        docs: [],
        pdf: [],
      },
    },
  },
);

export const useSizing = () => {
  const { props } = useModalState();

  if (props.inline) {
    return inlineClass({ type: props.type });
  } else {
    return popupClass({ type: props.type });
  }
};
