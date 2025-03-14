import {
  Card,
  Box,
  DataTable,
  Pagination,
  Text,
  Tooltip,
  ColumnContentType,
  IndexTable,
  IndexFilters,
  useSetIndexFiltersMode,
  TabProps,
  useBreakpoints,
  IndexFiltersProps,
} from "@shopify/polaris";
import { IndexTableHeading } from "@shopify/polaris/build/ts/src/components/IndexTable";
import { NonEmptyArray } from "@shopify/polaris/build/ts/src/types";


export interface Filter {
  key: string;
  label: string;
  filter: React.ReactNode;
}

export interface AdvancedTableComponentProps {
  data: any[][];
  page: number;
  setPage: (page: (page: number) => number) => void;
  tabs: string[];
  label: string;
  tableHeadings: { heading: string, tooltip: string }[];
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
          filters={filters.map((filter) => ({ ...filter, pinned: true, shortcut: true }))}
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
        />
        <IndexTable
          hasZebraStriping
          condensed={useBreakpoints().smDown}
          sortable={sortableColumns}
          onSort={onSort}
          selectable={false}
          itemCount={data.length}
          headings={
            tableHeadings.map((heading) => {
              return {
                title: (
                  <Tooltip content={heading.tooltip} hasUnderline>
                    <Text as="span" variant="bodyMd" fontWeight="bold">
                      {heading.heading}
                    </Text>
                  </Tooltip>
                ),
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
                {row.map((cell, index) => (
                  <IndexTable.Cell key={index}>{cell}</IndexTable.Cell>
                ))}
              </IndexTable.Row>
            );
          })}
        </IndexTable>
      </Box>
    </Card>
  );
};
