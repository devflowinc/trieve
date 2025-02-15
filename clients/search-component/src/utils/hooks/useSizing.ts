import { cva } from "cva";
import { useModalState } from "./modal-context";

const popupClass = cva(
  [
    "tv-max-h-[40vh]",
    "tv-min-w-[100vw]",
    "sm:tv-max-w-[800px]",
    "tv-top-[calc(40%-30vh)]",
    "tv-shadow-2xl tv-fixed",
    "tv-w-[90vw]",
    "tv-px-4",
  ],
  {
    variants: {
      type: {
        ecommerce: ["!tv-top-[0px]"],
        docs: ["tv-max-h-[80vh]"],
        pdf: ["tv-max-h-[70vh]"],
      },
      mode: {
        search: [],
        chat: ["!tv-max-h-[80vh]"],
      },
      modalPosition: {
        center: ["tv-left-[50%]", "-tv-translate-x-[50%]"],
        right: ["tv-right-[0px]"],
      },
    },
    compoundVariants: [
      {
        mode: "chat",
        type: "docs",
        className: "!tv-max-h-[80vh]",
      },
    ],

    defaultVariants: {
      modalPosition: "center",
    },
  },
);

const inlineClass = cva(
  [
    "tv-max-h-[40vh]",
    "sm:tv-max-w-[800px]",
    "!tv-min-w-full tv-max-w-sm tv-w-full",
    "tv-px-2",
  ],
  {
    variants: {
      type: {
        ecommerce: [
          "tv-top-1 tv-w-[95vw] tv-min-w-full tv-rounded-lg",
          "tv-px-0 tv-pt-0 tv-max-w-full",
        ],
        docs: [],
        pdf: [],
      },
    },
  },
);

export const useSizing = () => {
  const { props, mode } = useModalState();

  if (props.inline) {
    return inlineClass({ type: props.type });
  } else {
    return popupClass({
      type: props.type,
      mode: mode as "search" | "chat",
      modalPosition: props.modalPosition,
    });
  }
};
