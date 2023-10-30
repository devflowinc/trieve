import type { Setter } from "solid-js";
import { Show, createSignal, onMount } from "solid-js";
import type {
  CardMetadataWithVotes,
  CardCollectionDTO,
} from "../../utils/apiTypes";
import type { ScoreCardProps } from "./ScoreCard";
import { FiChevronLeft, FiChevronRight } from "solid-icons/fi";
import ScoreCard from "./ScoreCard";

export type ScoreCardAraryProps = Omit<
  ScoreCardProps,
  "card" | "counter" | "total" | "begin" | "end"
> & {
  cards: CardMetadataWithVotes[];
  setCardCollections: Setter<CardCollectionDTO[]>;
};

export const ScoreCardArray = (props: ScoreCardAraryProps) => {
  const dateValue =
    (import.meta.env.PUBLIC_DATE_RANGE_VALUE as string | undefined) ?? "Date";

  const [curCard, setCurCard] = createSignal(0);
  const [beginTime, setBeginTime] = createSignal<number | undefined>();
  const [endTime, setEndTime] = createSignal<number | undefined>();

  onMount(() => {
    props.cards.forEach((card) => {
      if (card.metadata && dateValue in card.metadata) {
        // regex to select only valid dates
        // (\d{1,4}([.\-/])\d{1,2}([.\-/])\d{1,4})
        const dateString = card.metadata[dateValue] as string;
        const dateRegex = /(\d{1,4}([.\-/])\d{1,2}([.\-/])\d{1,4})/;
        // extract the first match from the string
        const dateMatch = dateString.match(dateRegex)?.[0];
        if (!dateMatch) return;

        // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
        const dateObject = new Date(dateMatch);
        if (dateObject.getTime()) {
          setBeginTime((prev) =>
            Math.min(prev ?? Infinity, dateObject.getTime()),
          );
          setEndTime((prev) => Math.max(prev ?? 0, dateObject.getTime()));
        }
      }
    });
  });
  return (
    <div class="mx-auto flex max-w-[calc(100vw-32px)] items-center">
      <div class="w-[16px] min-[360px]:w-[32px]">
        <Show when={curCard() > 0}>
          <button onClick={() => setCurCard((prev) => prev - 1)}>
            <FiChevronLeft class="h-4 w-4 min-[360px]:h-8 min-[360px]:w-8" />
          </button>
        </Show>
        <Show when={curCard() <= 0}>
          <FiChevronLeft class="h-4 w-4 text-transparent min-[360px]:h-8 min-[360px]:w-8" />
        </Show>
      </div>
      <ScoreCard
        {...props}
        card={props.cards[curCard()]}
        counter={(curCard() + 1).toString()}
        total={props.cards.length}
        begin={beginTime()}
        end={endTime()}
        showExpand={props.showExpand}
      />
      <div class="w-[16px] min-[360px]:w-[32px]">
        <Show when={curCard() < props.cards.length - 1}>
          <button onClick={() => setCurCard((prev) => prev + 1)}>
            <FiChevronRight class="h-4 w-4 min-[360px]:h-8 min-[360px]:w-8" />
          </button>
        </Show>
        <Show when={curCard() >= props.cards.length - 1}>
          <FiChevronRight class="h-4 w-4 text-transparent min-[360px]:h-8 min-[360px]:w-8" />
        </Show>
      </div>
    </div>
  );
};
