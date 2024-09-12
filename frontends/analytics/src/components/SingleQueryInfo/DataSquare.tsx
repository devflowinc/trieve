export const DataSquare = (props: {
  label: string;
  value: number | string;
}) => {
  return (
    <div class="px-4 py-5 sm:p-6">
      <dt class="text-base font-normal text-gray-900">{props.label}</dt>
      <dd class="mt-1 flex items-baseline justify-start md:block lg:flex">
        <div class="flex items-baseline text-xl font-semibold text-fuchsia-600">
          {props.value}
        </div>
      </dd>
    </div>
  );
};
