import {
  Card,
  Box,
  Text,
  Tooltip,
  IndexTable,
  IndexFilters,
  useSetIndexFiltersMode,
  TabProps,
  useBreakpoints,
  IndexFiltersProps,
  Icon,
} from "@shopify/polaris";
import {
  Link
} from "@remix-run/react";
import { IndexTableHeading } from "@shopify/polaris/build/ts/src/components/IndexTable";
import { NonEmptyArray } from "@shopify/polaris/build/ts/src/types";
import {
  ChevronUpIcon,
  ChevronDownIcon,
} from '@shopify/polaris-icons';


export interface Filter {
  key: string;
  label: string;
  filter: React.ReactNode;
  pinned?: boolean;
}

export interface AdvancedTableCell {
  content: string;
  url?: string;
}

export interface AdvancedTableComponentProps {
  data: AdvancedTableCell[][];
  page: number;
  setPage: (page: (page: number) => number) => void;
  tabs: string[];
  label: string;
  tableHeadings: { heading: string, tooltip: string, sortCol?: string }[];
  sortOptions: IndexFiltersProps['sortOptions'];
  sortSelected: string[];
  setSortSelected: (sortSelected: string[]) => void;
  tooltipContent: string;
  hasNext: boolean;
  loading: boolean;
  appliedFilters: IndexFiltersProps['appliedFilters'];
  filters: Filter[];
  query: string;
  setQuery: (query: string) => void;
  handleClearAll: () => void;
  selected: number;
  setSelected: (selected: number) => void;
  sortableColumns?: boolean[];
  onSort?: (index: number, direction: "ascending" | "descending") => void;
  disableSearch?: boolean;
}

export const AdvancedTableComponent = ({
  data,
  page,
  setPage,
  label,
  tooltipContent,
  tabs,
  tableHeadings,
  hasNext,
  loading,
  sortableColumns,
  onSort,
  filters,
  query,
  setQuery,
  handleClearAll,
  selected,
  setSelected,
  appliedFilters,
  sortOptions,
  sortSelected,
  setSortSelected,
  disableSearch,
}: AdvancedTableComponentProps) => {
  const { mode, setMode } = useSetIndexFiltersMode();

  const indexTabs: TabProps[] = tabs.map((tab, index) => ({
    content: tab,
    index,
    id: `${tab}-${index}`,
    isLocked: index === 0,
  }));

  return (
    <Card>
      <div className="pb-2">
        <Tooltip content={tooltipContent} hasUnderline>
          <Text as="span" variant="bodyLg" fontWeight="bold">
            {label}
          </Text>
        </Tooltip>
      </div>
      <Box minHeight="14px">
        <IndexFilters
          sortOptions={sortOptions}
          sortSelected={sortSelected}
          onSort={setSortSelected}
          mode={mode}
          setMode={setMode}
          tabs={indexTabs}
          filters={filters.map((filter) => ({ ...filter, shortcut: true }))}
          queryValue={query}
          cancelAction={{
            onAction: () => { },
            disabled: false,
            loading: false,
          }}
          borderlessQueryField={false}
          appliedFilters={appliedFilters}
          onQueryChange={setQuery}
          onQueryClear={() => setQuery('')}
          onClearAll={handleClearAll}
          selected={selected}
          onSelect={setSelected}
          loading={loading}
          canCreateNewView={false}
          hideQueryField={disableSearch}
        />
        <IndexTable
          hasZebraStriping
          condensed={useBreakpoints().smDown}
          sortable={sortableColumns}
          onSort={onSort}
          selectable={false}
          itemCount={data.length}
          headings={
            tableHeadings.map((heading, index) => {
              return {
                title: (
                  <div onClick={() => {
                    if (heading.sortCol) {
                      setSortSelected([`${heading.sortCol} ${sortSelected[0].split(" ")[0] === heading.sortCol ? (sortSelected[0].split(" ")[1] === "asc" ? "desc" : "asc") : "asc"}`]);
                    }
                  }} className={`${heading.sortCol ? "cursor-pointer" : ""}`}>
                    <div className="flex flex-row items-center">
                      <Tooltip content={heading.tooltip} hasUnderline>
                        <Text as="span" variant="bodyMd" fontWeight="bold">
                          {heading.heading}
                        </Text>
                      </Tooltip>
                      {heading.sortCol && (
                        <span className="ml-1">{sortSelected[0].split(" ")[0] === heading.sortCol ? (sortSelected[0].split(" ")[1] === "asc" ? <Icon
                          source={ChevronUpIcon}
                          tone="base"
                        /> : <Icon
                          source={ChevronDownIcon}
                          tone="base"
                        />) : ""}
                        </span>
                      )}
                    </div>
                  </div>
                ),
                id: index.toString(),
              };
            }) as NonEmptyArray<IndexTableHeading>
          }
          pagination={
            {
              hasNext, hasPrevious: page > 1, onNext: () => {
                setPage((prevPage) => prevPage + 1);
              }, onPrevious: () => {
                setPage((prevPage) => prevPage - 1);
              }
            }
          }
        >
          {data.map((row, index) => {
            return (
              <IndexTable.Row key={index} id={index.toString()} position={index}>
                {row.map((cell, innerIndex) => (
                  <IndexTable.Cell
                    key={innerIndex}
                    className="max-w-[200px] truncate"
                  >
                    {cell.url ? (
                      <Link
                        to={cell.url}
                      >
                        {cell.content}
                      </Link>
                    ) : cell.content}
                  </IndexTable.Cell>
                ))}
              </IndexTable.Row>
            );
          })}
        </IndexTable>
      </Box>
    </Card>
  );
};
