import { Show, createSignal, onCleanup, onMount } from "solid-js";
import { Portal } from "solid-js/web";
import type { JSX } from "solid-js/jsx-runtime";

export type TooltipDirection = "top" | "bottom" | "left" | "right";

export interface TooltipProps {
  body?: JSX.Element;
  children?: JSX.Element;
  tooltipText: string;
  delay?: number;
  direction?: TooltipDirection;
  tooltipClass?: string;
  unsetWidth?: boolean;
}
const PADDING = 8; // Padding between tooltip and trigger

export const Tooltip = (props: TooltipProps) => {
  const [show, setShow] = createSignal(false);
  const [position, setPosition] = createSignal({ top: 0, left: 0 });
  let triggerRef: HTMLDivElement | undefined;
  let tooltipRef: HTMLDivElement | undefined;

  const calculateBestPosition = () => {
    if (!triggerRef || !tooltipRef) return null;

    const triggerRect = triggerRef.getBoundingClientRect();
    const tooltipRect = tooltipRef.getBoundingClientRect();

    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const scrollX = window.scrollX;
    const scrollY = window.scrollY;

    // Calculate available space in each direction
    const spaceAbove = triggerRect.top;
    const spaceBelow = viewportHeight - triggerRect.bottom;
    const spaceLeft = triggerRect.left;
    const spaceRight = viewportWidth - triggerRect.right;

    // Initialize positions for each direction
    const positions = {
      left: {
        top:
          triggerRect.top +
          scrollY +
          (triggerRect.height - tooltipRect.height) / 2,
        left: triggerRect.left + scrollX - tooltipRect.width - PADDING,
        space: spaceLeft,
      },
      right: {
        top:
          triggerRect.top +
          scrollY +
          (triggerRect.height - tooltipRect.height) / 2,
        left: triggerRect.right + scrollX + PADDING,
        space: spaceRight,
      },
      top: {
        top: triggerRect.top + scrollY - tooltipRect.height - PADDING,
        left:
          triggerRect.left +
          scrollX +
          (triggerRect.width - tooltipRect.width) / 2,
        space: spaceAbove,
      },
      bottom: {
        top: triggerRect.bottom + scrollY + PADDING,
        left:
          triggerRect.left +
          scrollX +
          (triggerRect.width - tooltipRect.width) / 2,
        space: spaceBelow,
      },
    };

    // Find the direction with the most space
    let bestDirection = "bottom"; // Default direction
    let maxSpace = positions.bottom.space;

    Object.entries(positions).forEach(([direction, pos]) => {
      if (pos.space > maxSpace) {
        maxSpace = pos.space;
        bestDirection = direction;
      }
    });

    // Get the chosen position
    let { top, left } = positions[bestDirection as keyof typeof positions];

    // Adjust if tooltip would go outside viewport
    if (left < scrollX) {
      left = scrollX + PADDING;
    } else if (left + tooltipRect.width > scrollX + viewportWidth) {
      left = scrollX + viewportWidth - tooltipRect.width - PADDING;
    }

    if (top < scrollY) {
      top = scrollY + PADDING;
    } else if (top + tooltipRect.height > scrollY + viewportHeight) {
      top = scrollY + viewportHeight - tooltipRect.height - PADDING;
    }

    return { top, left };
  };

  const updatePosition = () => {
    const newPosition = calculateBestPosition();
    if (newPosition) {
      setPosition(newPosition);
    }
  };

  onMount(() => {
    window.addEventListener("scroll", updatePosition);
    window.addEventListener("resize", updatePosition);
  });

  onCleanup(() => {
    window.removeEventListener("scroll", updatePosition);
    window.removeEventListener("resize", updatePosition);
  });

  return (
    <div class="relative">
      <div
        ref={triggerRef}
        class="cursor-help flex items-center"
        onMouseEnter={() => {
          setShow(true);
          setTimeout(updatePosition, 0);
        }}
        onMouseLeave={() => {
          setShow(false);
        }}
      >
        {props.children ? props.children : props.body}
      </div>
      <Show when={show()}>
        <Portal>
          <div
            ref={tooltipRef}
            class={props.tooltipClass}
            style={{
              position: "absolute",
              top: `${position().top}px`,
              left: `${position().left}px`,
              "z-index": "9999",
              opacity: "0",

              animation: `fadeIn 100ms ease-in forwards ${props.delay || 0}ms`,
            }}
            classList={{
              "inline-block rounded bg-neutral-100 border border-neutral-200 p-2 text-center shadow-lg dark:bg-neutral-800 dark:text-white dark:border-neutral-700 leading-snug text-black text-wrap":
                true,
              "w-[300px]": !props.unsetWidth,
            }}
          >
            {props.tooltipText}
          </div>
        </Portal>
      </Show>
    </div>
  );
};
