import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { allSearchesQuery } from "app/queries/analytics/search";
import { useEffect, useMemo, useState } from "react";
import {
  SearchAnalyticsFilter,
  SearchChunksReqPayload,
  SearchMethod,
  SearchSortBy,
  SearchType,
  SortOrder,
} from "trieve-ts-sdk";
import {
  formatStringDateRangeToDates,
  parseCustomDateString,
  toTitleCase,
  transformDateParams,
} from "app/utils/formatting";
import { AdvancedTableComponent, Filter } from "../AdvancedTableComponent";
import {
  Checkbox,
  ChoiceList,
  IndexFiltersProps,
  RangeSlider,
} from "@shopify/polaris";
import { DateRangePicker } from "../DateRangePicker";
import { ComponentNameSelect } from "../ComponentNameSelect";

export const AllSearchesTable = () => {
  const { trieve } = useTrieve();
  const [page, setPage] = useState(1);
  const [selected, setSelected] = useState(0);
  const [query, setQuery] = useState("");
  const [filters, setFilters] = useState<SearchAnalyticsFilter>({});
  const [hasClicks, setHasClicks] = useState<boolean | undefined>(undefined);
  const [appliedFilters, setAppliedFilters] = useState<
    IndexFiltersProps["appliedFilters"]
  >([]);
  const [sortSelected, setSortSelected] = useState<string[]>([
    "created_at desc",
  ]);
  const [sortBy, setSortBy] = useState<SearchSortBy | undefined>(undefined);
  const [sortOrder, setSortOrder] = useState<SortOrder | undefined>(undefined);
  const { data, isLoading } = useQuery(
    allSearchesQuery(trieve, filters, page, hasClicks, sortBy, sortOrder),
  );

  const client = useQueryClient();
  useEffect(() => {
    // prefetch the next page
    client.prefetchQuery(
      allSearchesQuery(trieve, filters, page + 1, hasClicks, sortBy, sortOrder),
    );
  }, [page, hasClicks, filters, sortBy, sortOrder]);

  const [previousData, setPreviousData] = useState<any[][]>([]);

  const mappedData = useMemo((): any[][] => {
    if (isLoading && previousData.length > 0) {
      return previousData;
    }

    if (!data?.queries) {
      return [];
    }

    let newData =
      data?.queries.map((query) => {
        const searchMethod = (query.request_params as SearchChunksReqPayload)
          .search_type;
        return [
          { content: query.query },
          { content: toTitleCase(query.search_type) },
          { content: toTitleCase(searchMethod) },
          { content: query.top_score },
          { content: query.latency },
          { content: query.query_rating ?? "N/A" },
          { content: query.results.length },
          { content: parseCustomDateString(query.created_at).toLocaleString() },
        ];
      }) ?? [];

    return newData;
  }, [data, hasClicks, filters]);

  useEffect(() => {
    if (mappedData.length > 0) {
      setPreviousData(mappedData);
    }
  }, [mappedData]);

  const sortOptions: IndexFiltersProps["sortOptions"] = [
    {
      label: "Queried At",
      value: "created_at asc",
      directionLabel: "Ascending",
    },
    {
      label: "Queried At",
      value: "created_at desc",
      directionLabel: "Descending",
    },
    { label: "Latency", value: "latency asc", directionLabel: "Ascending" },
    { label: "Latency", value: "latency desc", directionLabel: "Descending" },
    { label: "Top Score", value: "top_score asc", directionLabel: "Ascending" },
    {
      label: "Top Score",
      value: "top_score desc",
      directionLabel: "Descending",
    },
  ];

  const shopifyFilters: Filter[] = [
    {
      key: "component_name",
      label: "Component Name",
      filter: (
        <ComponentNameSelect
          filters={filters}
          setFilters={setFilters}
          onChange={(componentName) => {
            if (componentName != "") {
              setAppliedFilters([
                ...(appliedFilters?.filter(
                  (filter) => filter.key !== "component_name",
                ) || []),
                {
                  key: "component_name",
                  label: "Component Name: " + componentName,
                  onRemove: () => {
                    setFilters({
                      ...filters,
                      component_name: undefined,
                    });
                    setAppliedFilters(
                      appliedFilters?.filter(
                        (filter) => filter.key !== "component_name",
                      ),
                    );
                  },
                },
              ]);
            } else {
              setAppliedFilters(
                appliedFilters?.filter(
                  (filter) => filter.key !== "component_name",
                ),
              );
            }
          }}
        />
      ),
      pinned: true,
    },
    {
      key: "search_type",
      label: "Search Type",
      filter: (
        <ChoiceList
          title="Search Type"
          titleHidden
          choices={[
            { label: "All", value: "all" },
            { label: "Search", value: "search" },
            { label: "Autocomplete", value: "autocomplete" },
            { label: "Search Over Groups", value: "search_over_groups" },
            { label: "Search Within Groups", value: "search_within_groups" },
          ]}
          allowMultiple={false}
          selected={[filters.search_type || "all"]}
          onChange={(e) => {
            setFilters({
              ...filters,
              search_type: e[0] == "all" ? undefined : (e[0] as SearchType),
            });
            if (e[0] != "all") {
              setAppliedFilters([
                ...(appliedFilters?.filter(
                  (filter) => filter.key !== "search_type",
                ) || []),
                {
                  key: "search_type",
                  label: "Search Type: " + toTitleCase(e[0]),
                  onRemove: () => {
                    setFilters({
                      ...filters,
                      search_type: undefined,
                    });
                    setAppliedFilters(
                      appliedFilters?.filter(
                        (filter) => filter.key !== "search_type",
                      ),
                    );
                  },
                },
              ]);
            } else {
              setAppliedFilters(
                appliedFilters?.filter(
                  (filter) => filter.key !== "search_type",
                ),
              );
            }
          }}
        />
      ),
    },
    {
      key: "search_method",
      label: "Search Method",
      filter: (
        <ChoiceList
          title="Search Method"
          titleHidden
          choices={[
            { label: "All", value: "all" },
            { label: "Fulltext", value: "fulltext" },
            { label: "Hybrid", value: "hybrid" },
            { label: "Semantic", value: "semantic" },
          ]}
          allowMultiple={false}
          selected={[filters.search_method || "all"]}
          onChange={(e) => {
            setFilters({
              ...filters,
              search_method: e[0] == "all" ? undefined : (e[0] as SearchMethod),
            });
            if (e[0] != "all") {
              setAppliedFilters([
                ...(appliedFilters?.filter(
                  (filter) => filter.key !== "search_method",
                ) || []),
                {
                  key: "search_method",
                  label: "Search Method: " + toTitleCase(e[0]),
                  onRemove: () => {
                    setFilters({
                      ...filters,
                      search_method: undefined,
                    });
                    setAppliedFilters(
                      appliedFilters?.filter(
                        (filter) => filter.key !== "search_method",
                      ),
                    );
                  },
                },
              ]);
            } else {
              setAppliedFilters(
                appliedFilters?.filter(
                  (filter) => filter.key !== "search_method",
                ),
              );
            }
          }}
        />
      ),
      pinned: true,
    },
    {
      key: "date_range",
      label: "Date Range",
      filter: (
        <DateRangePicker
          value={formatStringDateRangeToDates(filters.date_range)}
          onChange={(e) => {
            setFilters({ ...filters, date_range: transformDateParams(e) });
            let label;

            if (e.gte == null) {
              label = "From All Time";
            } else if (
              e.gte?.getHours() === 0 &&
              e.gte?.getMinutes() === 0 &&
              e.gte?.getSeconds() === 0
            ) {
              label =
                "From " +
                e.gte?.toLocaleString("en-US", {
                  month: "short",
                  day: "numeric",
                  year: "numeric",
                }) +
                " to " +
                e.lte?.toLocaleString("en-US", {
                  month: "short",
                  day: "numeric",
                  year: "numeric",
                });
            } else {
              label =
                "From " +
                e.gte?.toLocaleString("en-US", {
                  month: "short",
                  day: "numeric",
                  year: "numeric",
                  hour: "numeric",
                  minute: "numeric",
                }) +
                " to " +
                e.lte?.toLocaleString("en-US", {
                  month: "short",
                  day: "numeric",
                  year: "numeric",
                  hour: "numeric",
                  minute: "numeric",
                });
            }
            setAppliedFilters([
              ...(appliedFilters?.filter(
                (filter) => filter.key !== "date_range",
              ) || []),
              {
                key: "date_range",
                label: label,
                onRemove: () => {
                  setFilters({
                    ...filters,
                    date_range: undefined,
                  });
                  setAppliedFilters(
                    appliedFilters?.filter(
                      (filter) => filter.key !== "date_range",
                    ),
                  );
                },
              },
            ]);
          }}
        />
      ),
      pinned: true,
    },
    {
      key: "query_rating",
      label: "Query Rating",
      filter: (
        <RangeSlider
          label="Query Rating"
          min={0}
          max={10}
          value={[filters.query_rating?.gt || 0, filters.query_rating?.lt || 5]}
          onChange={(e: [number, number]) => {
            setFilters({ ...filters, query_rating: { gte: e[0], lte: e[1] } });
            setAppliedFilters([
              ...(appliedFilters?.filter(
                (filter) => filter.key !== "query_rating",
              ) || []),
              {
                key: "query_rating",
                label: "Query Rating: " + e[0] + " - " + e[1],
                onRemove: () => {
                  setFilters({
                    ...filters,
                    query_rating: undefined,
                  });
                  setAppliedFilters(
                    appliedFilters?.filter(
                      (filter) => filter.key !== "query_rating",
                    ),
                  );
                },
              },
            ]);
          }}
        />
      ),
      pinned: true,
    },
    {
      key: "top_score",
      label: "Top Score",
      filter: (
        <RangeSlider
          label="Top Score"
          min={0}
          max={10}
          value={[filters.top_score?.gt || 0, filters.top_score?.lt || 5]}
          onChange={(e: [number, number]) => {
            setFilters({ ...filters, top_score: { gte: e[0], lte: e[1] } });
            setAppliedFilters([
              ...(appliedFilters?.filter(
                (filter) => filter.key !== "top_score",
              ) || []),
              {
                key: "top_score",
                label: "Top Score: " + e[0] + " - " + e[1],
                onRemove: () => {
                  setFilters({ ...filters, top_score: undefined });
                  setAppliedFilters(
                    appliedFilters?.filter(
                      (filter) => filter.key !== "top_score",
                    ),
                  );
                },
              },
            ]);
          }}
        />
      ),
      pinned: true,
    },
    {
      key: "has_clicks",
      label: "Has Clicks",
      filter: (
        <Checkbox
          label="Has Clicks"
          checked={hasClicks}
          onChange={(e) => {
            setHasClicks(e);
            setAppliedFilters([
              ...(appliedFilters?.filter(
                (filter) => filter.key !== "has_clicks",
              ) || []),
              {
                key: "has_clicks",
                label: "Has Clicks: " + e,
                onRemove: () => {
                  setHasClicks(undefined);
                  setAppliedFilters(
                    appliedFilters?.filter(
                      (filter) => filter.key !== "has_clicks",
                    ),
                  );
                },
              },
            ]);
          }}
        />
      ),
    },
  ];

  const tabs = [
    "All Searches",
    "Searches w/ Clicks",
    "Searches w/o Clicks",
    "Low Confidence Searches",
  ];

  useEffect(() => {
    if (selected === 0) {
      setHasClicks(undefined);
      setSortBy(undefined);
      setSortOrder(undefined);
      setFilters({});
      setAppliedFilters([]);
    } else if (selected === 1) {
      setHasClicks(true);
      setSortBy(undefined);
      setSortOrder(undefined);
      setFilters({});
      setAppliedFilters([]);
    } else if (selected === 2) {
      setHasClicks(false);
      setSortBy(undefined);
      setSortOrder(undefined);
      setFilters({});
      setAppliedFilters([]);
    } else if (selected === 3) {
      setHasClicks(undefined);
      setSortBy("top_score");
      setSortOrder("desc");
      setFilters({ top_score: { gte: 0, lte: 0.5 } });
      setAppliedFilters([
        {
          key: "top_score",
          label: "Top Score: 0 to 0.5",
          onRemove: () => {
            setFilters({ ...filters, top_score: undefined });
            setAppliedFilters(
              appliedFilters?.filter((filter) => filter.key !== "top_score"),
            );
          },
        },
      ]);
    }
  }, [selected]);

  useEffect(() => {
    if (sortSelected.length > 0) {
      const [sortBy, sortOrder] = sortSelected[0].split(" ");
      setSortBy(sortBy as SearchSortBy);
      setSortOrder(sortOrder as SortOrder);
    }
  }, [sortSelected]);

  useEffect(() => {
    if (query != "") {
      setFilters({
        ...filters,
        query: query,
      });
    } else {
      setFilters({
        ...filters,
        query: undefined,
      });
    }
  }, [query]);

  return (
    <AdvancedTableComponent
      data={mappedData}
      appliedFilters={appliedFilters}
      page={page}
      setPage={setPage}
      label="Searches"
      tableHeadings={[
        { heading: "Query", tooltip: "The search query" },
        {
          heading: "Search Type",
          tooltip:
            "Represents the type of search. Can be Search, Autocomplete, Search Over Groups, or Search Within Groups",
        },
        {
          heading: "Search Method",
          tooltip:
            "Represents the method used to search. Can be Fulltext, Hybrid, or Semantic",
        },
        { heading: "Top Score", tooltip: "The top score of the search" },
        { heading: "Latency", tooltip: "The latency of the search" },
        {
          heading: "Query Rating",
          tooltip: "The rating of the search by the user.",
        },
        { heading: "Results", tooltip: "The number of results returned" },
        {
          heading: "Created At",
          tooltip: "The date and time the search was made",
        },
      ]}
      hasNext={data?.queries.length == 10}
      filters={shopifyFilters}
      query={query}
      setQuery={setQuery}
      handleClearAll={() => {
        setQuery("");
        setFilters({});
        setAppliedFilters([]);
        setSortSelected(["created_at desc"]);
      }}
      selected={selected}
      setSelected={setSelected}
      loading={isLoading}
      sortOptions={sortOptions}
      sortSelected={sortSelected}
      setSortSelected={setSortSelected}
    />
  );
};
