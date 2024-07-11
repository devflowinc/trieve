import type { Accessor, Setter } from "solid-js";

export interface DatePickerProps {
  sectionName: string;
  timeRange: Accessor<{
    start: string;
    end: string;
  }>;
  setTimeRange: Setter<{
    start: string;
    end: string;
  }>;
  setPopoverOpen: (newState: boolean) => void;
}

export const DatePicker = (props: DatePickerProps) => {
  return (
    <div class="w-full min-w-[165px]">
      <div class="mb-1 text-center text-sm font-semibold">
        {props.sectionName}
      </div>
      <div class="mt-1 flex max-h-[40vh] w-full transform flex-col space-y-1 overflow-y-auto rounded px-2">
        <div class="flex flex-col items-center">
          <div class="relative">
            <input
              name="start"
              type="date"
              class="w-fit rounded-lg border border-gray-300 bg-gray-50 p-2 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-blue-500 dark:focus:ring-blue-500"
              value={props.timeRange().start}
              onChange={(e) => {
                props.setTimeRange((prev) => ({
                  ...prev,
                  start: e.currentTarget.value,
                }));
              }}
            />
          </div>
          <span class="mx-4 text-gray-500">to</span>
          <div class="relative pb-2">
            <input
              name="end"
              type="date"
              class="w-fit rounded-lg border border-gray-300 bg-gray-50 p-2 text-sm text-gray-900 focus:border-blue-500 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-blue-500 dark:focus:ring-blue-500"
              value={props.timeRange().end}
              onChange={(e) => {
                props.setTimeRange((prev) => ({
                  ...prev,
                  end: e.currentTarget.value,
                }));
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
};
