import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { useEffect, useMemo, useState } from "react";
import { RAGSortBy, SortOrder, TopicAnalyticsFilter, RagTypes } from "trieve-ts-sdk";
import { formatStringDateRangeToDates, parseCustomDateString, toTitleCase, transformDateParams } from "app/utils/formatting";
import { AdvancedTableComponent, Filter } from "../AdvancedTableComponent";
import { Checkbox, ChoiceList, IndexFiltersProps, RangeSlider } from "@shopify/polaris";
import { DateRangePicker } from "../DateRangePicker";
import { ComponentNameSelect } from "../ComponentNameSelect";
import { allChatsQuery } from "app/queries/analytics/chat";

export const AllChatsTable = () => {
  const { trieve } = useTrieve();
  const [page, setPage] = useState(1);
  const [selected, setSelected] = useState(0);
  const [query, setQuery] = useState("");
  const [filters, setFilters] = useState<TopicAnalyticsFilter>({});
  const [hasClicks, setHasClicks] = useState<boolean | undefined>(undefined);
  const [appliedFilters, setAppliedFilters] = useState<IndexFiltersProps['appliedFilters']>([]);
  const [sortSelected, setSortSelected] = useState<string[]>(['created_at desc']);
  const [sortBy, setSortBy] = useState<RAGSortBy | undefined>(undefined);
  const [sortOrder, setSortOrder] = useState<SortOrder | undefined>(undefined);
  const { data, isLoading } = useQuery(allChatsQuery(trieve, filters, page, hasClicks, sortBy, sortOrder));

  const client = useQueryClient();
  useEffect(() => {
    // prefetch the next page
    client.prefetchQuery(allChatsQuery(trieve, filters, page + 1, hasClicks, sortBy, sortOrder));
  }, [page, hasClicks, filters, sortBy, sortOrder]);

  const [previousData, setPreviousData] = useState<any[][]>([]);

  const mappedData = useMemo((): any[][] => {
    if (isLoading && previousData.length > 0) {
      console.log("returning previous data");
      return previousData;
    }

    if (!data?.topics) {
      return [];
    }

    let newData = data?.topics.map((query) => {
      return [
        query.name,
        query.message_count,
        query.avg_top_score,
        query.avg_hallucination_score,
        query.avg_query_rating ?? "N/A",
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
    { label: 'Hallucination Score', value: 'hallucination_score asc', directionLabel: 'Ascending' },
    { label: 'Hallucination Score', value: 'hallucination_score desc', directionLabel: 'Descending' },
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
      key: "rag_type",
      label: "RAG Type",
      filter: <ChoiceList
        title="RAG Type"
        titleHidden
        choices={[
          { label: "All", value: "all" },
          { label: "All Chunks", value: "all_chunks" },
          { label: "Selected Chunks", value: "chosen_chunks" },
        ]}
        allowMultiple={false}
        selected={[filters.rag_type || "all"]}
        onChange={(e) => {
          setFilters({
            ...filters,
            rag_type: e[0] == "all" ? undefined : e[0] as RagTypes,
          });
          if (e[0] != "all") {
            setAppliedFilters([...(appliedFilters?.filter((filter) => filter.key !== "rag_type") || []), {
              key: "rag_type",
              label: "RAG Type: " + toTitleCase(e[0]),
              onRemove: () => {
                setFilters({
                  ...filters,
                  rag_type: undefined,
                });
                setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "rag_type"));
              },
            }]);
          } else {
            setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "rag_type"));
          }
        }}
      />,
    },
    {
      key: "date_range",
      label: "Date Range",
      filter: <DateRangePicker
        value={formatStringDateRangeToDates(filters.date_range)}
        onChange={(e) => {
          setFilters({ ...filters, date_range: transformDateParams(e) });
          let label;
          if (e.gte?.getHours() === 0 &&
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
      key: "query_rating",
      label: "Query Rating",
      filter: <RangeSlider
        label="Query Rating"
        min={0}
        max={10}
        value={[filters.query_rating?.gt || 0, filters.query_rating?.lt || 5]}
        onChange={(e: [number, number]) => {
          setFilters({ ...filters, query_rating: { gte: e[0], lte: e[1] } });
          setAppliedFilters([...(appliedFilters?.filter((filter) => filter.key !== "query_rating") || []), {
            key: "query_rating",
            label: "Query Rating: " + e[0] + " - " + e[1],
            onRemove: () => {
              setFilters({
                ...filters,
                query_rating: undefined,
              });
              setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "query_rating"));
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
      key: "hallucination_score",
      label: "Hallucination Score",
      filter: <RangeSlider
        label="Hallucination Score"
        min={0}
        max={10}
        value={[filters.hallucination_score?.gt || 0 * 10, filters.hallucination_score?.lt || 5 * 10]}
        onChange={(e: [number, number]) => {
          setFilters({ ...filters, hallucination_score: { gte: e[0] / 10, lte: e[1] / 10 } });
          setAppliedFilters([...(appliedFilters?.filter((filter) => filter.key !== "hallucination_score") || []), {
            key: "hallucination_score",
            label: "Hallucination Score: " + (e[0] / 10) + " - " + (e[1] / 10),
            onRemove: () => {
              setFilters({ ...filters, hallucination_score: undefined });
              setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "hallucination_score"));
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

  const tabs = ["All Chats", "Chats w/ Clicks", "Chats w/o Clicks", "High Hallucination Score", "Low Top Score"];

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
      setAppliedFilters([{
        key: "has_clicks",
        label: "Has Clicks: true",
        onRemove: () => {
          setHasClicks(undefined);
          setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "has_clicks"));
        },
      }]);
    } else if (selected === 2) {
      setHasClicks(false);
      setSortBy(undefined);
      setSortOrder(undefined);
      setFilters({});
      setAppliedFilters([{
        key: "has_clicks",
        label: "Has Clicks: false",
        onRemove: () => {
          setHasClicks(undefined);
          setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "has_clicks"));
        },
      }]);
    } else if (selected === 3) {
      setHasClicks(undefined);
      setSortBy("hallucination_score");
      setSortOrder("desc");
      setFilters({ hallucination_score: { gte: 0.5, lte: 1 } });
      setAppliedFilters([{
        key: "hallucination_score",
        label: "Hallucination Score: 0.5 to 1",
        onRemove: () => {
          setFilters({ ...filters, hallucination_score: undefined });
          setAppliedFilters(appliedFilters?.filter((filter) => filter.key !== "hallucination_score"));
        },
      }]);
    } else if (selected === 4) {
      setHasClicks(undefined);
      setSortBy("top_score");
      setSortOrder("asc");
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
      setSortBy(sortBy as RAGSortBy);
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
      label="Chat Sessions"
      tooltipContent="View and filter all your users' chat sessions."
      tableHeadings={[
        { heading: "Name", tooltip: "The name created by the chatbot to represent the chat session." },
        { heading: "Message Count", tooltip: "The number of messages in the chat session." },
        { heading: "Avg Top Score", tooltip: "The average top score of the chat session." },
        { heading: "Avg Hallucination Score", tooltip: "The average hallucination score of the chat session." },
        { heading: "Avg Query Rating", tooltip: "The average query rating of the chat session." },
        { heading: "Created At", tooltip: "The date and time the chat session was created." },
      ]}
      hasNext={data?.topics.length == 10}
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
    />
  );
};
