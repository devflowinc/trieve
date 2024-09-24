import { BsArrowsCollapse } from "solid-icons/bs";
import { cva, VariantProps } from "cva";
import { createMemo, createSignal, For, Show, splitProps } from "solid-js";
import { z } from "zod";
import { cn } from "shared/utils";

interface QueryStringDisplayProps extends VariantProps<typeof queryStyles> {
  children: string;
}

const queryStyles = cva([""], {
  variants: {
    size: {
      default: "",
      large: "text-xl",
    },
  },
  defaultVariants: {
    size: "default",
  },
});

const weight = cva([""], {
  variants: {
    size: {
      default: "",
      large: "",
    },
  },
  defaultVariants: {
    size: "default",
  },
});

const multiQuerySchema = z.array(
  z.object({
    query: z.string(),
    weight: z.number(),
  }),
);

export const QueryStringDisplay = (props: QueryStringDisplayProps) => {
  const [children, others] = splitProps(props, ["children"]);

  const [isOpen, setIsOpen] = createSignal(false);
  const element = createMemo(() => {
    const query = children.children;
    if (query.startsWith("[{")) {
      // Attempt to parse as JSON array
      try {
        const object = JSON.parse(query) as unknown;
        const parseResult = multiQuerySchema.safeParse(object);
        if (parseResult.success) {
          if (parseResult.data.length === 1) {
            return parseResult?.data?.at(0)?.query;
          }
          return (
            <Show
              fallback={
                <div class="flex items-baseline gap-2">
                  <div class={queryStyles(others)}>
                    {parseResult.data.at(0)?.query}
                  </div>
                  <button
                    onClick={() => setIsOpen(true)}
                    class={cn(
                      "text-sm opacity-70 hover:underline",
                      weight(others),
                    )}
                  >
                    +{parseResult.data.length - 1} more..
                  </button>
                </div>
              }
              when={isOpen()}
            >
              <div class="flex items-center gap-2">
                <div>
                  <For each={parseResult.data}>
                    {(query, index) => (
                      <div class="flex items-baseline gap-2">
                        <div>{query.query}</div>
                        <div class="text-sm text-neutral-500">
                          {query.weight.toFixed(2)}
                        </div>
                        <Show when={index() === 0}>
                          <button
                            onClick={() => setIsOpen(false)}
                            class="relative top-[3px] cursor-pointer px-1 opacity-70"
                          >
                            <BsArrowsCollapse />
                          </button>
                        </Show>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            </Show>
          );
        } else {
          return query;
        }
      } catch {
        return query;
      }
    }
    return query;
  });

  return (
    <div>
      <div>{element()}</div>
    </div>
  );
};
