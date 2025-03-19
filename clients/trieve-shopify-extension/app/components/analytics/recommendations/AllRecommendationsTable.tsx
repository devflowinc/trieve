import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { useEffect, useMemo, useState } from "react";
import { RecommendationAnalyticsFilter, RecommendationSortBy, SortOrder } from "trieve-ts-sdk";
import { formatStringDateRangeToDates, parseCustomDateString, transformDateParams } from "app/utils/formatting";
import { AdvancedTableComponent, Filter } from "../AdvancedTableComponent";
import { Checkbox, IndexFiltersProps, RangeSlider } from "@shopify/polaris";
import { DateRangePicker } from "../DateRangePicker";
import { ComponentNameSelect } from "../ComponentNameSelect";
import { allRecommendationsQuery } from "app/queries/analytics/recommendation";
export const AllRecommendationsTable = () => {
  const { trieve } = useTrieve();
  const [page, setPage] = useState(1);
  const [selected, setSelected] = useState(0);
  const [query, setQuery] = useState("");
  const [filters, setFilters] = useState<RecommendationAnalyticsFilter>({});
  const [hasClicks, setHasClicks] = useState<boolean | undefined>(undefined);
  const [appliedFilters, setAppliedFilters] = useState<IndexFiltersProps['appliedFilters']>([]);
  const [sortSelected, setSortSelected] = useState<string[]>(['created_at desc']);
  const [sortBy, setSortBy] = useState<RecommendationSortBy | undefined>(undefined);
  const [sortOrder, setSortOrder] = useState<SortOrder | undefined>(undefined);
  const { data, isLoading } = useQuery(allRecommendationsQuery(trieve, filters, page, hasClicks, sortBy, sortOrder));

  const client = useQueryClient();
  useEffect(() => {
    // prefetch the next page
    client.prefetchQuery(allRecommendationsQuery(trieve, filters, page + 1, hasClicks, sortBy, sortOrder));
  }, [page, hasClicks, filters, sortBy, sortOrder]);

  const [previousData, setPreviousData] = useState<any[][]>([]);

  const mappedData = useMemo((): any[][] => {
    if (isLoading && previousData.length > 0) {
      console.log("returning previous data");
      return previousData;
    }

    if (!data?.queries) {
      return [];
    }

    let newData = data?.queries.map((query) => {
      return [
        query.positive_tracking_ids,
        query.negative_tracking_ids,
        query.top_score,
        query.created_at,
        query.results.length,
        parseCustomDateString(query.created_at).toLocaleString(),
      ];
    }) ?? [];

    return newData;
  }, [data, hasClicks, filters]);

  useEffect(() => {
    if (mappedData.length > 0) {
      setPreviousData(mappedData);
    }
  }, [mappedData]);

  const sortOptions: IndexFiltersProps['sortOptions'] = [
    { label: 'Queried At', value: 'created_at asc', directionLabel: 'Ascending' },
    { label: 'Queried At', value: 'created_at desc', directionLabel: 'Descending' },
    { label: 'Top Score', value: 'top_score asc', directionLabel: 'Ascending' },
    { label: 'Top Score', value: 'top_score desc', directionLabel: 'Descending' },
  ];


  const shopifyFilters: Filter[] = [
    {
      key: "component_name",
      label: "Component Name",
      filter: <ComponentNameSelect filters={filters} setFilters={setFilters} onChange={(componentName) => {
        if (componentName != "") {
          setAppliedFilters([...(appliedFilters?.filter((filter) => filter.key !== "component_name") || []), {
            key: "component_name",
            label: "Component Name: " + componentName,
            onRemove: () => {
              setFilters({
                ...filters,
                component_name: undefined,
              });
              setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "component_name"));
            },
          }]);
        } else {
          setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "component_name"));
        }
      }} />,
      pinned: true,
    },
    {
      key: "date_range",
      label: "Date Range",
      filter: <DateRangePicker
        value={formatStringDateRangeToDates(filters.date_range)}
        onChange={(e) => {
          setFilters({ ...filters, date_range: transformDateParams(e) });
          let label;

          if (e.gte == null) {
            label = "From All Time";
          } else if (e.gte?.getHours() === 0 &&
            e.gte?.getMinutes() === 0 &&
            e.gte?.getSeconds() === 0) {
            label = "From " + e.gte?.toLocaleString("en-US", {
              month: "short",
              day: "numeric",
              year: "numeric",
            }) + " to " + e.lte?.toLocaleString("en-US", {
              month: "short",
              day: "numeric",
              year: "numeric",
            });
          } else {
            label = "From " + e.gte?.toLocaleString("en-US", {
              month: "short",
              day: "numeric",
              year: "numeric",
              hour: "numeric",
              minute: "numeric",
            }) + " to " + e.lte?.toLocaleString("en-US", {
              month: "short",
              day: "numeric",
              year: "numeric",
              hour: "numeric",
              minute: "numeric",
            });
          }
          setAppliedFilters([...(appliedFilters?.filter((filter) => filter.key !== "date_range") || []), {
            key: "date_range",
            label: label,
            onRemove: () => {
              setFilters({
                ...filters,
                date_range: undefined,
              });
              setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "date_range"));
            },
          }]);
        }}
      />,
      pinned: true,
    },
    {
      key: "top_score",
      label: "Top Score",
      filter: <RangeSlider
        label="Top Score"
        min={0}
        max={10}
        value={[filters.top_score?.gt || 0, filters.top_score?.lt || 5]}
        onChange={(e: [number, number]) => {
          setFilters({ ...filters, top_score: { gte: e[0], lte: e[1] } });
          setAppliedFilters([...(appliedFilters?.filter((filter) => filter.key !== "top_score") || []), {
            key: "top_score",
            label: "Top Score: " + e[0] + " - " + e[1],
            onRemove: () => {
              setFilters({ ...filters, top_score: undefined });
              setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "top_score"));
            },
          }]);
        }}
      />,
      pinned: true,
    },
    {
      key: "has_clicks",
      label: "Has Clicks",
      filter: <Checkbox label="Has Clicks" checked={hasClicks} onChange={(e) => {
        setHasClicks(e);
        setAppliedFilters([...(appliedFilters?.filter((filter) => filter.key !== "has_clicks") || []), {
          key: "has_clicks",
          label: "Has Clicks: " + e,
          onRemove: () => {
            setHasClicks(undefined);
            setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "has_clicks"));
          },
        }]);
      }} />,
    },
  ];

  const tabs = ["All Recommendations", "Recommendations w/ Clicks", "Recommendations w/o Clicks", "Low Confidence Recommendations"];

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
      setAppliedFilters([{
        key: "top_score",
        label: "Top Score: 0 to 0.5",
        onRemove: () => {
          setFilters({ ...filters, top_score: undefined });
          setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "top_score"));
        },
      }]);
    }
  }, [selected]);

  useEffect(() => {
    if (sortSelected.length > 0) {
      const [sortBy, sortOrder] = sortSelected[0].split(" ");
      setSortBy(sortBy as RecommendationSortBy);
      setSortOrder(sortOrder as SortOrder);
    }
  }, [sortSelected]);


  return (
    <AdvancedTableComponent
      data={mappedData}
      appliedFilters={appliedFilters}
      page={page}
      setPage={setPage}
      label="Recommendations"
      tooltipContent="View and filter all your recommendations."
      tableHeadings={[
        { heading: "Positive Tracking IDs", tooltip: "The positive tracking ids of the recommendation" },
        { heading: "Negative Tracking IDs", tooltip: "The negative tracking ids of the recommendation" },
        { heading: "Top Score", tooltip: "The top score of the recommendation" },
        { heading: "Created At", tooltip: "The date and time the recommendation was created" },
        { heading: "Results", tooltip: "The number of results returned" }
      ]}
      hasNext={data?.queries.length == 10}
      tabs={tabs}
      filters={shopifyFilters}
      query={query}
      setQuery={setQuery}
      handleClearAll={() => {
        setQuery("");
        setFilters({});
        setAppliedFilters([]);
        setSortSelected(['created_at desc']);
      }}
      selected={selected}
      setSelected={setSelected}
      loading={isLoading}
      sortOptions={sortOptions}
      sortSelected={sortSelected}
      setSortSelected={setSortSelected}
      disableSearch={true}
    />
  );
};
