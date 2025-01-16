import { SearchQueryEvent } from "shared/types";
import { parseCustomDateString } from "../utils/formatDate";
import { toTitleCase } from "../utils/titleCase";
interface SearchQueryEventModalProps {
  searchEvent: SearchQueryEvent;
}
export const SearchQueryEventModal = (props: SearchQueryEventModalProps) => {
  return (
    <div class="min-w-60 pt-4">
      <SmallCol
        value={parseCustomDateString(
          props.searchEvent.created_at,
        ).toLocaleString()}
        label="Searched At"
      />
      <SmallCol
        value={props.searchEvent.results.length}
        label="Results Obtained"
      />
      <SmallCol
        value={toTitleCase(props.searchEvent.search_type)}
        label="Search Type"
      />
      <SmallCol value={props.searchEvent.latency + "ms"} label="Latency" />
      <SmallCol value={props.searchEvent.top_score} label="Top Score" />
    </div>
  );
};

interface SmallColProps {
  label: string;
  value: string | number;
}
const SmallCol = (props: SmallColProps) => {
  return (
    <div class="flex items-center justify-between gap-8">
      <div class="text-neutral-500">{props.label}</div>
      <div class="text-neutral-700">{props.value}</div>
    </div>
  );
};
