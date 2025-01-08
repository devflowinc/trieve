import { Show, For, createSignal, createEffect } from "solid-js";

export interface FieldFilter {
  field: string;
  geo_radius?: {
    center?: {
      lat?: number | null;
      lon?: number | null;
    } | null;
    radius?: number | null;
  } | null;
  match_any?: (string | number)[] | null;
  match_all?: (string | number)[] | null;
  range?: {
    gte?: number | null;
    lte?: number | null;
    gt?: number | null;
    lt?: number | null;
  } | null;
  boolean?: boolean | null;
  date_range?: {
    gte?: string | null;
    lte?: string | null;
    gt?: string | null;
    lt?: string | null;
  } | null;
}

export interface HasIdFilter {
  ids: string[] | null;
  tracking_ids: string[] | null;
}

export type Filter = FieldFilter | HasIdFilter;

export interface Filters {
  must: Filter[];
  must_not: Filter[];
  should: Filter[];
}

export function filterIsHasIdFilter(filter: Filter): filter is HasIdFilter {
  return (
    filter != null &&
    ((filter as HasIdFilter)["ids"] != null ||
      (filter as HasIdFilter)["tracking_ids"] != null)
  );
}

export function filterAsHasIdFilter(filter: Filter): HasIdFilter | null {
  return filterIsHasIdFilter(filter) ? filter : null;
}

export function filterIsFieldFilter(filter: Filter): filter is FieldFilter {
  return filter != null && !filterIsHasIdFilter(filter);
}

export function filterAsFieldFilter(filter: Filter): FieldFilter | null {
  return filterIsFieldFilter(filter) ? filter : null;
}

export interface FilterItemProps {
  initialFilter: Filter;
  onFilterChange: (filter: Filter) => void;
}

export const FilterItem = (props: FilterItemProps) => {
  const [curFilter, setCurFilter] = createSignal<Filter>(props.initialFilter);

  let initialTempFilterMode = "match_any";
  let initialTempFilterField = "tag_set";

  if (filterIsFieldFilter(props.initialFilter) && props.initialFilter != null) {
    const fieldMode: ["geo_radius", "range", "boolean", "date_range"] = [
      "geo_radius",
      "range",
      "boolean",
      "date_range",
    ];

    for (const attempt of fieldMode)
      if (props.initialFilter[attempt] != null) {
        initialTempFilterMode = attempt;
        break;
      }
  } else if (
    filterIsHasIdFilter(props.initialFilter) &&
    props.initialFilter != null
  ) {
    const fieldMode: ["tracking_ids", "ids"] = ["tracking_ids", "ids"];

    for (const attempt of fieldMode)
      if (props.initialFilter[attempt] != null) {
        initialTempFilterMode = "has_id_filter";
        initialTempFilterField = attempt;
        break;
      }
  }

  const [tempFilterMode, setTempFilterMode] = createSignal<string>(
    initialTempFilterMode,
  );

  const [tempFilterField, setTempFilterField] = createSignal<string>(
    filterAsFieldFilter(props.initialFilter)?.field ?? initialTempFilterField,
  );
  const [location, setLocation] = createSignal({
    lat:
      filterAsFieldFilter(props.initialFilter)?.geo_radius?.center?.lat ?? null,
    lon:
      filterAsFieldFilter(props.initialFilter)?.geo_radius?.center?.lon ?? null,
    radius:
      filterAsFieldFilter(props.initialFilter)?.geo_radius?.radius ?? null,
  });

  const [range, setRange] = createSignal({
    gt: filterAsFieldFilter(props.initialFilter)?.range?.gt ?? null,
    lt: filterAsFieldFilter(props.initialFilter)?.range?.lt ?? null,
    gte: filterAsFieldFilter(props.initialFilter)?.range?.gte ?? null,
    lte: filterAsFieldFilter(props.initialFilter)?.range?.lte ?? null,
  });

  const [dateRange, setDateRange] = createSignal({
    gt: filterAsFieldFilter(props.initialFilter)?.date_range?.gt ?? null,
    lt: filterAsFieldFilter(props.initialFilter)?.date_range?.lt ?? null,
    gte: filterAsFieldFilter(props.initialFilter)?.date_range?.gte ?? null,
    lte: filterAsFieldFilter(props.initialFilter)?.date_range?.lte ?? null,
  });

  const [boolean, setBoolean] = createSignal<boolean | null>(
    filterAsFieldFilter(props.initialFilter)?.boolean ?? null,
  );

  const [matchAny, setMatchAny] = createSignal<(string | number)[] | null>(
    filterAsFieldFilter(props.initialFilter)?.match_any ?? null,
  );

  const [matchAll, setMatchAll] = createSignal<(string | number)[] | null>(
    filterAsFieldFilter(props.initialFilter)?.match_all ?? null,
  );

  const [idFilterText, setIdFilterText] = createSignal<string[]>(
    filterAsHasIdFilter(props.initialFilter)?.tracking_ids ??
      filterAsHasIdFilter(props.initialFilter)?.ids ??
      [],
  );

  createEffect(() => {
    const changedField = tempFilterField();
    const curFieldFilter = filterAsFieldFilter(props.initialFilter);

    if (changedField === "ids") {
      setCurFilter({
        ids: idFilterText(),
        tracking_ids: null,
      } as HasIdFilter);
    } else if (changedField === "tracking_ids") {
      setCurFilter({
        ids: null,
        tracking_ids: idFilterText(),
      } as HasIdFilter);
      filterIsHasIdFilter(curFilter());
    } else {
      if (curFieldFilter?.match_all?.length) {
        setTempFilterMode("match_all");
      } else if (curFieldFilter?.match_any?.length) {
        setTempFilterMode("match_any");
      } else if (
        curFieldFilter?.geo_radius?.center ||
        curFieldFilter?.geo_radius?.radius
      ) {
        setTempFilterMode("geo_radius");
      } else if (curFieldFilter?.range?.gt || curFieldFilter?.range?.lt) {
        setTempFilterMode("range");
      } else if (curFieldFilter?.boolean != null) {
        setTempFilterMode("boolean");
      } else if (
        curFieldFilter?.date_range?.gt ||
        curFieldFilter?.date_range?.lt
      ) {
        setTempFilterMode("date_range");
      }
      return;
    }
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
        match_any: null,
        match_all: null,
        range: null,
        date_range: null,
      });
      setMatchAll(null);
      setMatchAny(null);
      setRange({
        gt: null,
        lt: null,
        gte: null,
        lte: null,
      });
      setDateRange({
        gt: null,
        lt: null,
        gte: null,
        lte: null,
      });
    }

    if (changedMode === "range") {
      setCurFilter({
        field: tempFilterField(),
        match_any: null,
        match_all: null,
        range: {
          gt: range().gt,
          lt: range().lt,
          gte: range().gte,
          lte: range().lte,
        },
        date_range: null,
      });
      setMatchAll(null);
      setMatchAny(null);
      setDateRange({
        gt: null,
        lt: null,
        gte: null,
        lte: null,
      });
      setLocation({
        lat: null,
        lon: null,
        radius: null,
      });
    }

    if (changedMode === "boolean") {
      setCurFilter({
        field: tempFilterField(),
        match_any: null,
        match_all: null,
        range: null,
        date_range: null,
        boolean: boolean(),
      });
      setMatchAll(null);
      setMatchAny(null);
      setRange({
        gt: null,
        lt: null,
        gte: null,
        lte: null,
      });
      setDateRange({
        gt: null,
        lt: null,
        gte: null,
        lte: null,
      });
      setLocation({
        lat: null,
        lon: null,
        radius: null,
      });
    }

    if (changedMode === "match_any") {
      setCurFilter({
        field: tempFilterField(),
        match_any: matchAny(),
        match_all: null,
        range: null,
        date_range: null,
      });
      setMatchAll(null);
      setRange({
        gt: null,
        lt: null,
        gte: null,
        lte: null,
      });
      setDateRange({
        gt: null,
        lt: null,
        gte: null,
        lte: null,
      });
      setLocation({
        lat: null,
        lon: null,
        radius: null,
      });
    }

    if (changedMode === "match_all") {
      console.log("match_all");
      setCurFilter({
        field: tempFilterField(),
        match_any: null,
        match_all: matchAll(),
        range: null,
        date_range: null,
      } satisfies FieldFilter);
      setMatchAny(null);
      setRange({
        gt: null,
        lt: null,
        gte: null,
        lte: null,
      });
      setDateRange({
        gt: null,
        lt: null,
        gte: null,
        lte: null,
      });
      setLocation({
        lat: null,
        lon: null,
        radius: null,
      });
    }

    if (changedMode === "date_range") {
      setCurFilter({
        field: tempFilterField(),
        match_any: null,
        match_all: null,
        range: null,
        date_range: {
          gt: dateRange().gt,
          lt: dateRange().lt,
          gte: dateRange().gte,
          lte: dateRange().lte,
        },
      });
      setMatchAll(null);
      setMatchAny(null);
      setRange({
        gt: null,
        lt: null,
        gte: null,
        lte: null,
      });
      setLocation({
        lat: null,
        lon: null,
        radius: null,
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
              : tempFilterField().startsWith("group_metadata")
                ? "group_metadata"
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
              "group_tracking_ids",
              "group_ids",
              "group_metadata",
              "tracking_ids",
              "ids",
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
        <Show when={tempFilterField().startsWith("group_metadata")}>
          <div>
            <span class="p-2">.</span>
            <input
              type="text"
              placeholder="field"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setTempFilterField("group_metadata." + e.currentTarget.value);
              }}
              value={tempFilterField()
                .replace("group_metadata", "")
                .replace(".", "")}
            />
          </div>
        </Show>
      </div>
      <Show when={filterIsFieldFilter(curFilter())}>
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
            <For
              each={[
                "match_any",
                "match_all",
                "geo_radius",
                "range",
                "boolean",
                "date_range",
              ]}
            >
              {(filterMode) => {
                return (
                  <option
                    classList={{
                      "flex w-full items-center justify-between rounded p-1":
                        true,
                      "bg-neutral-300 dark:bg-neutral-900":
                        filterMode === tempFilterMode(),
                    }}
                  >
                    {filterMode}
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
                onInput={(e) => {
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
                onInput={(e) => {
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
                onInput={(e) => {
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
                onInput={(e) => {
                  setRange({
                    ...range(),
                    gt: parseFloat(e.currentTarget.value),
                  });
                }}
                value={range().gt ?? ""}
              />
              <label aria-label="Less Than">Less Than:</label>
              <input
                type="number"
                placeholder="Less Than"
                class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
                onInput={(e) => {
                  setRange({
                    ...range(),
                    lt: parseFloat(e.currentTarget.value),
                  });
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
                onInput={(e) => {
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
                onInput={(e) => {
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
        <Show when={tempFilterMode() === "boolean"}>
          <div class="flex flex-col gap-y-1 py-2 pl-4">
            <div class="grid grid-cols-2 items-center">
              <label aria-label="Match Any">Boolean:</label>
              <select
                class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
                onChange={(e) => {
                  setBoolean(e.currentTarget.value === "true");
                }}
                value={boolean() ? "true" : "false"}
              >
                <option>true</option>
                <option>false</option>
              </select>
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
                onInput={(e) => {
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
                onInput={(e) => {
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
                onInput={(e) => {
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
                onInput={(e) => {
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
        <Show when={tempFilterMode() === "match_any"}>
          <div class="flex flex-col gap-y-1 py-2 pl-4">
            <div class="grid grid-cols-2 items-center">
              <label aria-label="Match Any">Comma Separated Values:</label>
              <input
                type="text"
                placeholder="h1, h2, h3"
                class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
                onInput={(e) => {
                  setMatchAny(e.currentTarget.value.split(","));
                }}
                value={(matchAny() ?? []).join(",")}
              />
            </div>
          </div>
        </Show>
        <Show when={tempFilterMode() === "match_all"}>
          <div class="flex flex-col gap-y-1 py-2 pl-4">
            <div class="grid grid-cols-2 items-center">
              <label aria-label="Match All">Comma Separated Values:</label>
              <input
                type="text"
                placeholder="h1, h2, h3"
                class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
                onInput={(e) => {
                  setMatchAll(e.currentTarget.value.split(","));
                }}
                value={(matchAll() ?? []).join(",")}
              />
            </div>
          </div>
        </Show>
      </Show>

      <Show when={filterIsHasIdFilter(curFilter())}>
        <div class="flex flex-col gap-y-1 py-2 pl-4">
          <div class="grid grid-cols-2 items-center">
            <label aria-label="Value">Comma Separated Values:</label>
            <input
              type="text"
              placeholder="h1, h2, h3"
              class="rounded-md border border-neutral-400 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
              onChange={(e) => {
                setIdFilterText(e.currentTarget.value.split(","));
              }}
              value={idFilterText()}
            />
          </div>
        </div>
      </Show>
    </div>
  );
};
