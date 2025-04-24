import {
  Card,
  Box,
  Text,
  Tooltip,
  IndexTable,
  IndexFilters,
  useSetIndexFiltersMode,
  useBreakpoints,
  IndexFiltersProps,
  Icon,
  Page,
  InlineStack,
} from "@shopify/polaris";
import { Link } from "@remix-run/react";
import { IndexTableHeading } from "@shopify/polaris/build/ts/src/components/IndexTable";
import { NonEmptyArray } from "@shopify/polaris/build/ts/src/types";
import {
  ChevronUpIcon,
  ChevronDownIcon,
  ViewIcon,
} from "@shopify/polaris-icons";
import { ReactNode } from "react";

export interface Filter {
  key: string;
  label: string;
  filter: React.ReactNode;
  pinned?: boolean;
}

// Either component or content is required
export interface AdvancedTableCell {
  component?: React.ReactNode;
  content?: string;
  url?: string;
}

export interface AdvancedTableComponentProps {
  data: AdvancedTableCell[][];
  page: number;
  setPage: (page: (page: number) => number) => void;
  label: string;
  tableHeadings: { heading: string; tooltip: string; sortCol?: string }[];
  sortOptions: IndexFiltersProps["sortOptions"];
  sortSelected: string[];
  setSortSelected: (sortSelected: string[]) => void;
  hasNext: boolean;
  loading: boolean;
  appliedFilters: IndexFiltersProps["appliedFilters"];
  filters: Filter[];
  query: string;
  setQuery: (query: string) => void;
  handleClearAll: () => void;
  selected: number;
  setSelected: (selected: number) => void;
  sortableColumns?: boolean[];
  onSort?: (index: number, direction: "ascending" | "descending") => void;
  disableSearch?: boolean;
  additionalControls?: ReactNode;
}

export const AdvancedTableComponent = ({
  data,
  page,
  setPage,
  label,
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
  additionalControls,
}: AdvancedTableComponentProps) => {
  const { mode, setMode } = useSetIndexFiltersMode();

  return (
    <Page title={label} fullWidth={true}>
      <Card>
        <Box minHeight="14px">
          <div className="flex justify-end">
            <div className="grow border-b border-b-neutral-200 flex items-end pb-1">
              {additionalControls}
            </div>
            <div className="shrink-0">
              <IndexFilters
                sortOptions={sortOptions}
                sortSelected={sortSelected}
                onSort={setSortSelected}
                mode={mode}
                setMode={setMode}
                tabs={[]}
                filters={filters.map((filter) => ({
                  ...filter,
                  shortcut: true,
                }))}
                queryValue={query}
                cancelAction={{
                  onAction: () => {},
                  disabled: false,
                  loading: false,
                }}
                borderlessQueryField={false}
                appliedFilters={appliedFilters}
                onQueryChange={setQuery}
                onQueryClear={() => setQuery("")}
                onClearAll={handleClearAll}
                selected={selected}
                onSelect={setSelected}
                loading={loading}
                canCreateNewView={false}
                hideQueryField={disableSearch}
              />
            </div>
          </div>
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
                    <div
                      onClick={() => {
                        if (heading.sortCol) {
                          setSortSelected([
                            `${heading.sortCol} ${sortSelected[0].split(" ")[0] === heading.sortCol ? (sortSelected[0].split(" ")[1] === "asc" ? "desc" : "asc") : "asc"}`,
                          ]);
                        }
                      }}
                      className={`${heading.sortCol ? "cursor-pointer" : ""}`}
                    >
                      <div className="flex flex-row items-center">
                        <Tooltip content={heading.tooltip} hasUnderline>
                          <Text as="span" variant="bodyMd" fontWeight="bold">
                            {heading.heading}
                          </Text>
                        </Tooltip>
                        {heading.sortCol && (
                          <span className="ml-1">
                            {sortSelected[0].split(" ")[0] ===
                            heading.sortCol ? (
                              sortSelected[0].split(" ")[1] === "asc" ? (
                                <Icon source={ChevronUpIcon} tone="base" />
                              ) : (
                                <Icon source={ChevronDownIcon} tone="base" />
                              )
                            ) : (
                              ""
                            )}
                          </span>
                        )}
                      </div>
                    </div>
                  ),
                  id: index.toString(),
                };
              }) as NonEmptyArray<IndexTableHeading>
            }
            pagination={{
              hasNext,
              hasPrevious: page > 1,
              onNext: () => {
                setPage((prevPage) => prevPage + 1);
              },
              onPrevious: () => {
                setPage((prevPage) => prevPage - 1);
              },
            }}
          >
            {data.map((row, index) => {
              return (
                <IndexTable.Row
                  key={index}
                  id={index.toString()}
                  position={index}
                >
                  {row.map((cell, innerIndex) => {
                    const innerCell = cell.component ? (
                      cell.component
                    ) : (
                      <Text as="span" variant="bodyMd" truncate={true}>
                        {cell.content}
                      </Text>
                    );
                    return (
                      <IndexTable.Cell
                        key={innerIndex}
                        className="max-w-[250px] truncate"
                      >
                        {cell.url ? (
                          <Link to={cell.url}>
                            <InlineStack wrap={false}>
                              <ViewIcon width={20} />
                              {innerCell}
                            </InlineStack>
                          </Link>
                        ) : (
                          innerCell
                        )}
                      </IndexTable.Cell>
                    );
                  })}
                </IndexTable.Row>
              );
            })}
          </IndexTable>
        </Box>
      </Card>
    </Page>
  );
};
