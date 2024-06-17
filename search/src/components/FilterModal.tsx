/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import {
  Accessor,
  Show,
  For,
  createSignal,
  createEffect,
  useContext,
  createMemo,
} from "solid-js";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";

export interface Filter {
  field: string;
  geo_radius?: {
    center?: {
      lat?: number | null;
      lon?: number | null;
    } | null;
    radius?: number | null;
  } | null;
  match?: (string | number)[] | null;
  range?: {
    gte?: number | null;
    lte?: number | null;
    gt?: number | null;
    lt?: number | null;
  } | null;
  date_range?: {
    gte?: string | null;
    lte?: string | null;
    gt?: string | null;
    lt?: string | null;
  } | null;
}

export interface Filters {
  must: Filter[];
  must_not: Filter[];
  should: Filter[];
}

export interface FilterModalProps {
  showFilterModal: Accessor<boolean>;
  setShowFilterModal: (open: boolean) => void;
}

const defaultFilter = {
  field: "",
};

export const FilterModal = (props: FilterModalProps) => {
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const [tempFilterType, setTempFilterType] = createSignal<string>("must");
  const [mustFilters, setMustFilters] = createSignal<Filter[]>([]);
  const [mustNotFilters, setMustNotFilters] = createSignal<Filter[]>([]);
  const [shouldFilters, setShouldFilters] = createSignal<Filter[]>([]);

  const curDatasetFiltersKey = createMemo(
    () =>
      `filters-${datasetAndUserContext.currentDataset?.()?.dataset.id ?? ""}`,
  );

  const saveFilters = () => {
    const filters = {
      must: mustFilters(),
      must_not: mustNotFilters(),
      should: shouldFilters(),
    };
    localStorage.setItem(curDatasetFiltersKey(), JSON.stringify(filters));
    window.dispatchEvent(new Event("filtersUpdated"));
    props.setShowFilterModal(false);
  };

  createEffect((prevFiltersKey) => {
    const filtersKey = curDatasetFiltersKey();
    if (prevFiltersKey === filtersKey) {
      return filtersKey;
    }

    const savedFilters = localStorage.getItem(filtersKey);
    if (savedFilters) {
      const parsedFilters = JSON.parse(savedFilters) as Filters;
      setMustFilters(parsedFilters.must);
      setMustNotFilters(parsedFilters.must_not);
      setShouldFilters(parsedFilters.should);
    }
  }, "");

  return (
    <div class="flex max-h-[50vh] min-w-[70vw] max-w-[75vw] flex-col space-y-2 overflow-auto px-2 pr-2 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-thumb-rounded-md dark:text-white dark:scrollbar-track-neutral-800 dark:scrollbar-thumb-neutral-600 xl:min-w-[50vw] 2xl:min-w-[40vw]">
      <div class="flex w-full items-center space-x-2 border-b border-neutral-400 py-2 dark:border-neutral-900">
        <label aria-label="Change Filter Type">
          <span class="p-1">Filter Type:</span>
        </label>
        <select
          class="h-fit rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
          onChange={(e) => {
            setTempFilterType(e.currentTarget.value);
          }}
          value={tempFilterType()}
        >
          <For each={["must", "must not", "should"]}>
            {(filter_type) => {
              return (
                <option
                  classList={{
                    "flex w-full items-center justify-between rounded p-1":
                      true,
                    "bg-neutral-300 dark:bg-neutral-900":
                      filter_type === tempFilterType(),
                  }}
                >
                  {filter_type}
                </option>
              );
            }}
          </For>
        </select>
        <button
          class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
          onClick={() => {
            const curFilterType = tempFilterType();
            if (curFilterType === "must") {
              setMustFilters([...mustFilters(), defaultFilter]);
            }
            if (curFilterType === "must not") {
              setMustNotFilters([...mustNotFilters(), defaultFilter]);
            }
            if (curFilterType === "should") {
              setShouldFilters([...shouldFilters(), defaultFilter]);
            }
          }}
        >
          + Add Filter
        </button>
        <div class="flex-1" />
        <button
          class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
          onClick={() => {
            setMustFilters([]);
            setMustNotFilters([]);
            setShouldFilters([]);
          }}
        >
          Reset Filters
        </button>
        <button
          class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
          onClick={() => saveFilters()}
        >
          Apply Filters
        </button>
      </div>
      <Show when={mustFilters().length > 0}>
        <div class="border-b border-neutral-400 py-2 dark:border-neutral-900">
          must: [
          <div class="flex flex-col gap-y-2">
            <For each={mustFilters()}>
              {(filter, index) => {
                const onFilterChange = (newFilter: Filter) => {
                  const newFilters = mustFilters();
                  newFilters[index()] = newFilter;
                  setMustFilters(newFilters);
                };

                return (
                  <div
                    classList={{
                      "border-b border-dotted border-neutral-400 dark:border-neutral-900":
                        index() < mustFilters().length - 1,
                    }}
                  >
                    <FilterItem
                      initialFilter={filter}
                      onFilterChange={onFilterChange}
                    />
                  </div>
                );
              }}
            </For>
          </div>
          ]
        </div>
      </Show>
      <Show when={mustNotFilters().length > 0}>
        <div class="border-b border-neutral-400 py-2 dark:border-neutral-900">
          must not: [
          <div class="flex flex-col gap-y-2">
            <For each={mustNotFilters()}>
              {(filter, index) => {
                const onFilterChange = (newFilter: Filter) => {
                  const newFilters = mustNotFilters();
                  newFilters[index()] = newFilter;
                  setMustNotFilters(newFilters);
                };

                return (
                  <div
                    classList={{
                      "border-b border-dotted border-neutral-400 dark:border-neutral-900":
                        index() < mustNotFilters().length - 1,
                    }}
                  >
                    <FilterItem
                      initialFilter={filter}
                      onFilterChange={onFilterChange}
                    />
                  </div>
                );
              }}
            </For>
          </div>
          ]
        </div>
      </Show>
      <Show when={shouldFilters().length > 0}>
        <div class="border-b border-neutral-400 py-2 dark:border-neutral-900">
          should: [
          <div class="flex flex-col gap-y-2">
            <For each={shouldFilters()}>
              {(filter, index) => {
                const onFilterChange = (newFilter: Filter) => {
                  const newFilters = shouldFilters();
                  newFilters[index()] = newFilter;
                  setShouldFilters(newFilters);
                };

                return (
                  <div
                    classList={{
                      "border-b border-dotted border-neutral-400 dark:border-neutral-900":
                        index() < shouldFilters().length - 1,
                    }}
                  >
                    <FilterItem
                      initialFilter={filter}
                      onFilterChange={onFilterChange}
                    />
                  </div>
                );
              }}
            </For>
          </div>
          ]
        </div>
      </Show>
    </div>
  );
};

export interface FilterItemProps {
  initialFilter?: Filter;
  onFilterChange: (filter: Filter) => void;
}

export const FilterItem = (props: FilterItemProps) => {
  const [curFilter, setCurFilter] = createSignal<Filter>(
    props.initialFilter ?? defaultFilter,
  );
  const [tempFilterMode, setTempFilterMode] = createSignal<string>(
    props.initialFilter?.geo_radius
      ? "geo_radius"
      : props.initialFilter?.range
        ? "range"
        : props.initialFilter?.date_range
          ? "date_range"
          : props.initialFilter?.match
            ? "match"
            : "match",
  );
  const [tempFilterField, setTempFilterField] = createSignal<string>(
    props.initialFilter?.field ?? "",
  );
  const [location, setLocation] = createSignal({
    lat: props.initialFilter?.geo_radius?.center?.lat ?? null,
    lon: props.initialFilter?.geo_radius?.center?.lon ?? null,
    radius: props.initialFilter?.geo_radius?.radius ?? null,
  });

  const [range, setRange] = createSignal({
    gt: props.initialFilter?.range?.gt ?? null,
    lt: props.initialFilter?.range?.lt ?? null,
    gte: props.initialFilter?.range?.gte ?? null,
    lte: props.initialFilter?.range?.lte ?? null,
  });

  const [dateRange, setDateRange] = createSignal({
    gt: props.initialFilter?.date_range?.gt ?? null,
    lt: props.initialFilter?.date_range?.lt ?? null,
    gte: props.initialFilter?.date_range?.gte ?? null,
    lte: props.initialFilter?.date_range?.lte ?? null,
  });

  const [match, setMatch] = createSignal<(string | number)[] | null>(
    props.initialFilter?.match ?? null,
  );

  createEffect(() => {
    const changedMode = tempFilterMode();
    setCurFilter({
      field: tempFilterField(),
      [changedMode]: {},
    });
  });

  createEffect(() => {
    const changedMode = tempFilterMode();
    if (changedMode === "geo_radius") {
      setCurFilter({
        field: tempFilterField(),
        geo_radius: {
          center: {
            lat: location().lat,
            lon: location().lon,
          },
          radius: location().radius,
        },
      });
    }

    if (changedMode === "range") {
      setCurFilter({
        field: tempFilterField(),
        range: {
          gt: range().gt,
          lt: range().lt,
          gte: range().gte,
          lte: range().lte,
        },
      });
    }

    if (changedMode === "match") {
      setCurFilter({
        field: tempFilterField(),
        match: match(),
      });
    }

    if (changedMode === "date_range") {
      setCurFilter({
        field: tempFilterField(),
        date_range: {
          gt: dateRange().gt,
          lt: dateRange().lt,
          gte: dateRange().gte,
          lte: dateRange().lte,
        },
      });
    }
  });

  createEffect(() => {
    props.onFilterChange(curFilter());
  });

  return (
    <div class="flex flex-col gap-y-2 py-1">
      <div class="flex items-center gap-y-1">
        <label aria-label="Change Filter Field">
          <span class="p-1">Filter Field:</span>
        </label>
        <select
          class="h-fit w-48 rounded-md border border-neutral-400 bg-neutral-100 p-1 pl-1 dark:border-neutral-900 dark:bg-neutral-800"
          onChange={(e) => {
            setTempFilterField(e.currentTarget.value);
          }}
          value={
            tempFilterField().startsWith("metadata")
              ? "metadata"
              : tempFilterField()
          }
        >
          <For
            each={[
              "tag_set",
              "link",
              "time_stamp",
              "location",
              "metadata",
              "num_value",
            ]}
          >
            {(filter_field) => {
              return (
                <option
                  classList={{
                    "flex w-full items-center justify-between rounded p-1":
                      true,
                    "bg-neutral-300 dark:bg-neutral-900":
                      tempFilterField().startsWith(filter_field),
                  }}
                >
                  {filter_field}
                </option>
              );
            }}
          </For>
        </select>
        <Show when={tempFilterField().startsWith("metadata")}>
          <div>
            <span class="p-2">.</span>
            <input
              type="text"
              placeholder="field"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setTempFilterField("metadata." + e.currentTarget.value);
              }}
              value={tempFilterField().replace("metadata", "").replace(".", "")}
            />
          </div>
        </Show>
      </div>
      <div class="w-full">
        <label aria-label="Change Filter Mode">
          <span class="p-1">Filter Mode:</span>
        </label>
        <select
          class="h-fit w-48 rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
          onChange={(e) => {
            setTempFilterMode(e.currentTarget.value);
          }}
          value={tempFilterMode()}
        >
          <For each={["match", "geo_radius", "range", "date_range"]}>
            {(filter_mode) => {
              return (
                <option
                  classList={{
                    "flex w-full items-center justify-between rounded p-1":
                      true,
                    "bg-neutral-300 dark:bg-neutral-900":
                      filter_mode === tempFilterMode(),
                  }}
                >
                  {filter_mode}
                </option>
              );
            }}
          </For>
        </select>
      </div>
      <Show when={tempFilterMode() === "geo_radius"}>
        <div class="flex flex-col gap-y-1 py-2 pl-2">
          <div class="grid w-full grid-cols-2 items-center gap-y-1">
            <label aria-label="Latitude">Latitude:</label>
            <input
              type="number"
              placeholder="Latitude"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setLocation({
                  ...location(),
                  lat: parseFloat(e.currentTarget.value),
                });
              }}
              value={location().lat ?? ""}
            />
          </div>
          <div class="grid w-full grid-cols-2 items-center gap-y-1">
            <label aria-label="Longitude">Longitude:</label>
            <input
              type="number"
              placeholder="Longitude"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setLocation({
                  ...location(),
                  lon: parseFloat(e.currentTarget.value),
                });
              }}
              value={location().lon ?? ""}
            />
          </div>
          <div class="grid w-full grid-cols-2 items-center gap-y-1">
            <label aria-label="Radius">Radius:</label>
            <input
              type="number"
              placeholder="Radius"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setLocation({
                  ...location(),
                  radius: parseFloat(e.currentTarget.value),
                });
              }}
              value={location().radius ?? ""}
            />
          </div>
        </div>
      </Show>
      <Show when={tempFilterMode() === "range"}>
        <div class="flex w-full flex-col gap-y-1 pl-2">
          <div class="grid w-full grid-cols-2 items-center gap-y-1">
            <label aria-label="Greater Than">Greater Than:</label>
            <input
              type="number"
              placeholder="Greater Than"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setRange({ ...range(), gt: parseFloat(e.currentTarget.value) });
              }}
              value={range().gt ?? ""}
            />
            <label aria-label="Less Than">Less Than:</label>
            <input
              type="number"
              placeholder="Less Than"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setRange({ ...range(), lt: parseFloat(e.currentTarget.value) });
              }}
              value={range().lt ?? ""}
            />
          </div>
          <div class="grid w-full grid-cols-2 items-center gap-y-1">
            <label aria-label="Greater Than or Equal">
              Greater Than or Equal:
            </label>
            <input
              type="number"
              placeholder="Greater Than or Equal"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setRange({
                  ...range(),
                  gte: parseFloat(e.currentTarget.value),
                });
              }}
              value={range().gte ?? ""}
            />
            <label aria-label="Less Than or Equal">Less Than or Equal:</label>
            <input
              type="number"
              placeholder="Less Than or Equal"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setRange({
                  ...range(),
                  lte: parseFloat(e.currentTarget.value),
                });
              }}
              value={range().lte ?? ""}
            />
          </div>
        </div>
      </Show>
      <Show when={tempFilterMode() === "date_range"}>
        <div class="flex w-full flex-col gap-y-1 pl-2">
          <div class="grid w-full grid-cols-2 items-center gap-y-1">
            <label aria-label="Greater Than">Greater Than:</label>
            <input
              type="date"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setDateRange({
                  ...dateRange(),
                  gt: e.currentTarget.value + " 00:00:00",
                });
              }}
              value={dateRange().gt?.replace(" 00:00:00", "") ?? ""}
            />
            <label aria-label="Less Than">Less Than:</label>
            <input
              type="date"
              placeholder="Less Than"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setDateRange({
                  ...dateRange(),
                  lt: e.currentTarget.value + " 00:00:00",
                });
              }}
              value={dateRange().lt?.replace(" 00:00:00", "") ?? ""}
            />
          </div>
          <div class="grid w-full grid-cols-2 items-center gap-y-1">
            <label aria-label="Greater Than or Equal">
              Greater Than or Equal:
            </label>
            <input
              type="date"
              placeholder="Greater Than or Equal"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setDateRange({
                  ...dateRange(),
                  gte: e.currentTarget.value + " 00:00:00",
                });
              }}
              value={dateRange().gte?.replace(" 00:00:00", "") ?? ""}
            />
            <label aria-label="Less Than or Equal">Less Than or Equal:</label>
            <input
              type="date"
              placeholder="Less Than or Equal"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setDateRange({
                  ...dateRange(),
                  lte: e.currentTarget.value + " 00:00:00",
                });
              }}
              value={dateRange().lte?.replace(" 00:00:00", "") ?? ""}
            />
          </div>
        </div>
      </Show>
      <Show when={tempFilterMode() === "match"}>
        <div class="flex flex-col gap-y-1 py-2 pl-4">
          <div class="grid grid-cols-2 items-center">
            <label aria-label="Match">Comma Separated Values:</label>
            <input
              type="text"
              placeholder="h1, h2, h3"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setMatch(e.currentTarget.value.split(","));
              }}
              value={(match() ?? []).join(",")}
            />
          </div>
        </div>
      </Show>
    </div>
  );
};
