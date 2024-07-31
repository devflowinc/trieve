import DatePicker, {
  DateObjectUnits,
  DatePickerOnChange,
  PickerAloneValue,
  PickerValue,
} from "@rnwonder/solid-date-picker";
import "@rnwonder/solid-date-picker/dist/style.css";
import "./datePickerStyles.css";
import utils from "@rnwonder/solid-date-picker/utilities";
import type { AnalyticsParams, DateRangeFilter } from "../types";
import { Accessor, createMemo, createSignal, For, Show } from "solid-js";
import { TbSelector } from "solid-icons/tb";
import {
  differenceInHours,
  formatDistanceToNowStrict,
  getDate,
  subHours,
  subMinutes,
} from "date-fns";
import { differenceInMinutes } from "date-fns/fp";

const getGranularitySuggestion = (
  range: DateRangeFilter,
): AnalyticsParams["granularity"] => {
  const startDate = range.gt || range.gte;
  const endDate = range.lt || range.lte || new Date();
  if (!startDate) {
    // Ideally this doesn't happen
    return "hour";
  }

  const minDifference = differenceInMinutes(startDate, endDate);
  if (minDifference < 3) {
    return "second";
  }
  // Get the number of hours inbetween
  const difference = differenceInHours(endDate, startDate);
  if (difference < 3) {
    return "minute";
  } else if (difference <= 24) {
    return "hour";
  }
  return "day";
};

const transformOurTypeToTheirs = (value: DateRangeFilter): PickerAloneValue => {
  const startDate = value.gt || value.gte;
  if (!startDate) {
    throw new Error("DateRangeFilter must have a gt or gte value");
  }
  let endDate = value.lt || value.lte;
  if (!endDate) {
    endDate = new Date();
  }

  return {
    startDateObject: utils().convertDateToDateObject(startDate),
    endDateObject: utils().convertDateToDateObject(endDate),
  };
};

const transformTheirTypeToOurs = (
  startDate: DateObjectUnits,
  endDate: DateObjectUnits,
): DateRangeFilter => {
  const actuallyNow = unitIsToday(endDate);
  return {
    gte: utils().convertDateObjectToDate(startDate),
    lte: actuallyNow ? new Date() : utils().convertDateObjectToDate(endDate),
    gt: undefined,
    lt: undefined,
  };
};

const getLabelFromRange = (value: DateRangeFilter): string => {
  const endDate = value.lte || value.lt || new Date();
  const startDate = value.gte || value.gt;
  if (!startDate) {
    return "Error"; // This should never happen
  }
  // The end range of the date is set to right now, use simple labels
  if (Math.abs(differenceInMinutes(endDate, new Date())) < 4) {
    return "Last " + formatDistanceToNowStrict(startDate);
  }

  return "NO LABEL";
};

export const unitIsToday = (unit: DateObjectUnits) => {
  const today = new Date();
  if (
    unit.day == getDate(today) &&
    unit.month == today.getMonth() &&
    unit.year == today.getFullYear()
  ) {
    return true;
  }
  return false;
};

interface DateRangePickerProps {
  label?: string;
  value: DateRangeFilter;
  onChange: (value: DateRangeFilter) => void;
  onGranularitySuggestion?: (
    granularity: AnalyticsParams["granularity"],
  ) => void;
  initialSelectedPresetId?: number;
  presets?: DatePreset[];
}

export const DateRangePicker = (props: DateRangePickerProps) => {
  const [selectedPresetId, setSelectedPresetId] = createSignal<number>(
    props.initialSelectedPresetId || 0,
  );

  const dateRangeValue = createMemo(() => {
    const transformed = transformOurTypeToTheirs(props.value);
    return {
      value: transformed,
      label: getLabelFromRange(props.value),
    } satisfies PickerValue;
  });

  const handleChange: ((data: DatePickerOnChange) => void) | undefined = (
    data,
  ) => {
    if (data.type === "range") {
      if (data.startDate && data.endDate) {
        setSelectedPresetId(0);
        const transformed = transformTheirTypeToOurs(
          data.startDate,
          data.endDate,
        );

        props.onChange(transformed);
        console.log("post transform");
        if (props.onGranularitySuggestion) {
          const newGranularity = getGranularitySuggestion(transformed);
          console.log("Granularity suggestion", newGranularity);
          props.onGranularitySuggestion(newGranularity);
        }
      }
    } else {
      console.log("Impossible date range change selected");
      return;
    }
  };

  const element: HTMLDivElement | undefined = undefined;

  const close = () => {
    document.getElementsByClassName("date-picker-wrapper").item(0)?.remove();
  };

  return (
    <div>
      <DatePicker
        ref={element}
        daysActiveRangeBetweenWrapperClass="daysActiveRangeBetweenWrapperClass"
        shouldCloseOnSelect={true}
        value={dateRangeValue}
        monthSelectorType="compact-dropdown"
        onChange={handleChange}
        maxDate={utils().convertDateToDateObject(new Date())}
        calendarLeftAreaJSX={
          <Presets
            onGranularitySuggestion={props.onGranularitySuggestion}
            presets={props.presets}
            onChange={(range) => {
              props.onChange(range);
              close();
            }}
            value={props.value}
            selectedPresetId={selectedPresetId()}
            setSelectedPresetId={setSelectedPresetId}
          />
        }
        primaryColor="#A33EB5"
        textColor="#262626"
        renderInput={({ value, showDate }) => (
          <CustomInput value={value} showDate={showDate} label={props.label} />
        )}
        type="range"
      />
    </div>
  );
};

const CustomInput = (props: {
  label?: string;
  value: Accessor<PickerValue>;
  showDate: () => void;
}) => {
  return (
    <div class="">
      <Show when={props.label}>
        <div class="text-sm text-neutral-600">{props.label}</div>
      </Show>
      <button
        onClick={() => props.showDate()}
        class="flex truncate bg-white rounded border border-neutral-300 py-1 text-sm px-3 w-full justify-between gap-2 items-center"
      >
        {props.value().label || "No Time Range Selected"}
        <TbSelector />
      </button>
    </div>
  );
};

type DatePreset = {
  id?: number;
  label: string;
  range: DateRangeFilter;
  granularity: AnalyticsParams["granularity"];
};

const defaultDatePresets: DatePreset[] = [
  {
    id: 1,
    label: "Last 15 Minutes",
    range: {
      gt: undefined,
      lt: undefined,
      lte: undefined,
      gte: subMinutes(new Date(), 15),
    },
    granularity: "minute",
  },
  {
    id: 2,
    label: "Last 30 Minutes",
    range: {
      gt: undefined,
      lt: undefined,
      lte: undefined,
      gte: subMinutes(new Date(), 30),
    },
    granularity: "minute",
  },
  {
    id: 3,
    label: "Last Hour",
    range: {
      gt: undefined,
      lt: undefined,
      lte: undefined,
      gte: subHours(new Date(), 1),
    },
    granularity: "minute",
  },
  {
    id: 4,
    label: "Last 3 Hours",
    range: {
      gt: undefined,
      lt: undefined,
      lte: undefined,
      gte: subHours(new Date(), 3),
    },
    granularity: "hour",
  },
  {
    id: 5,
    label: "Last 12 Hours",
    range: {
      gt: undefined,
      lt: undefined,
      lte: undefined,
      gte: subHours(new Date(), 12),
    },
    granularity: "hour",
  },
  {
    id: 6,
    label: "Last 24 Hours",
    range: {
      gt: undefined,
      lt: undefined,
      lte: undefined,
      gte: subHours(new Date(), 24),
    },
    granularity: "hour",
  },
];

interface PresetsProps {
  value: DateRangeFilter;
  presets?: DatePreset[];
  onChange: (value: DateRangeFilter) => void;
  selectedPresetId: number;
  setSelectedPresetId: (id: number) => void;
  onGranularitySuggestion?: (
    granularity: AnalyticsParams["granularity"],
  ) => void;
}

const Presets = (props: PresetsProps) => {
  return (
    <div class="min-w-[158px] border-r-neutral-200 my-2 border-r h-full">
      <div>
        <For each={props.presets || defaultDatePresets}>
          {(preset) => (
            <button
              classList={{
                "block text-left pl-4 w-full font-semibold text-sm p-1 hover:bg-neutral-100":
                  true,
                "bg-magenta-100 hover:bg-magenta-200":
                  props.selectedPresetId === preset.id,
              }}
              onClick={() => {
                if (preset.id) {
                  props.setSelectedPresetId(preset.id);
                }
                props.onChange(preset.range);
                if (props.onGranularitySuggestion) {
                  props.onGranularitySuggestion(preset.granularity);
                }
              }}
            >
              {preset.label}
            </button>
          )}
        </For>
      </div>
    </div>
  );
};
