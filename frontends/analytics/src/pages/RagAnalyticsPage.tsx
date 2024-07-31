import { RequiredRAGAnalyticsFilter } from "shared/types";
import { createStore } from "solid-js/store";
import { createEffect, createSignal } from "solid-js";
import { DateRangePicker, Select } from "shared/ui";
import { RagQueries } from "../components/charts/RagQueries";
import { RagUsage } from "../components/charts/RagUsage";
import { subDays } from "date-fns";
import { RAGUsageGraph } from "../components/charts/RAGUsageGraph";

type FakeRAGType = "chosen_chunks" | "all_chunks" | "both";
type FakeRAGOption = {
  label: string;
  value: FakeRAGType;
};
const ALL_FAKE_RAG_OPTIONS: FakeRAGOption[] = [
  { label: "Both", value: "both" },
  { label: "Chosen chunks", value: "chosen_chunks" },
  { label: "All chunks", value: "all_chunks" },
];

export const RagAnalyticsPage = () => {
  const [filter, setFilter] = createStore<RequiredRAGAnalyticsFilter>({
    date_range: {
      gt: subDays(new Date(), 7),
    },
    rag_type: undefined,
    granularity: "day",
  });

  const [fakeType, setFakeType] = createSignal<FakeRAGOption>(
    ALL_FAKE_RAG_OPTIONS[0],
  );

  // Sync the fake rag type with the real one, setting it undefined if its both
  createEffect(() => {
    if (fakeType().value === "both") {
      setFilter("rag_type", undefined);
    } else {
      setFilter(
        "rag_type",
        fakeType().value as RequiredRAGAnalyticsFilter["rag_type"],
      );
    }
  });

  return (
    <>
      <div class="flex justify-between border-neutral-400 px-3 py-2">
        <div>
          <Select
            class="min-w-[200px] !bg-white"
            label={<div class="text-sm text-neutral-600">RAG Type</div>}
            display={(option) => option.label}
            options={ALL_FAKE_RAG_OPTIONS}
            selected={fakeType()}
            onSelected={setFakeType}
          />
        </div>
        <div>
          <DateRangePicker
            label="Date Range"
            value={filter.date_range}
            onChange={(e) => setFilter("date_range", e)}
            onGranularitySuggestion={(granularity) =>
              setFilter("granularity", granularity)
            }
          />
        </div>
      </div>
      <div class="grid grid-cols-2 gap-4 p-2 pt-3">
        <RagQueries filter={filter} />
        <RagUsage filter={filter} />
        <RAGUsageGraph
          params={{
            filter,
            granularity: filter.granularity,
          }}
        />
      </div>
    </>
  );
};
