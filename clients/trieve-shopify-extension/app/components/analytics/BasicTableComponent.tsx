import {
  Card,
  Box,
  DataTable,
  Pagination,
  Text,
  Tooltip,
  ColumnContentType,
} from "@shopify/polaris";

export interface TableComponentProps {
  data: any[][];
  page: number;
  setPage: (page: (page: number) => number) => void;
  label?: string;
  tableContentTypes: ColumnContentType[];
  tableHeadings: string[];
  tooltipContent?: string;
  hasNext: boolean;
  sortableColumns?: boolean[];
  onSort?: (index: number, direction: "ascending" | "descending") => void;
  hidePagination?: boolean;
  noCard?: boolean;
}

export const BasicTableComponent = ({
  data,
  page,
  setPage,
  label,
  tooltipContent,
  tableContentTypes,
  tableHeadings,
  hasNext,
  sortableColumns,
  onSort,
  hidePagination,
  noCard = false,
}: TableComponentProps) => {
  const tableContent = (
    <>
      <div className="pb-2">
        {label && (
          <Tooltip content={tooltipContent} hasUnderline>
            <Text as="span" variant="bodyLg" fontWeight="bold">
              {label}
            </Text>
          </Tooltip>
        )}
      </div>
      <Box minHeight="14px">
        <DataTable
          truncate
          hasZebraStripingOnData
          sortable={sortableColumns}
          onSort={onSort}
          rows={data}
          columnContentTypes={tableContentTypes}
          headings={tableHeadings}
        />
        {!hidePagination && (
          <div className="flex justify-end mt-2">
            <Pagination
              type="page"
              onNext={() => {
                setPage((prevPage) => prevPage + 1);
              }}
              onPrevious={() => {
                setPage((prevPage) => prevPage - 1);
              }}
              hasPrevious={page > 1}
              hasNext={hasNext}
            ></Pagination>
          </div>
        )}
      </Box>
    </>
  );
  return noCard ? (
    <div className="w-full">{tableContent}</div>
  ) : (
    <Card>{tableContent}</Card>
  );
};
