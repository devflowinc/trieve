import { JSX } from "solid-js";

interface Props {
  children?: JSX.Element;
}

// Needs to ensure dataset and org don't desync
export const DatasetLayout = (props: Props) => {
  return (
    <div class="grid max-h-full grow grid-cols-[200px_1fr] overflow-hidden">
      <div class="h-full bg-red-500">sidebar</div>
      <div class="overflow-scroll p-4">{props.children}</div>
    </div>
  );
};
