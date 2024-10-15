import { JSX } from "solid-js";
export const DataSquare = (props: {
  label: string;
  value: number | string | JSX.Element;
  onClick?: () => void;
}) => {
  return (
    <div
      class="px-4 py-5 sm:p-6"
      onClick={() => {
        if (props.onClick) {
          props.onClick();
        }
      }}
    >
      <dt class="text-base font-normal text-gray-900">{props.label}</dt>
      <dd class="mt-1 flex items-baseline justify-start md:block lg:flex">
        <div class="flex items-baseline text-xl font-semibold text-magenta-600">
          {props.value}
        </div>
      </dd>
    </div>
  );
};
