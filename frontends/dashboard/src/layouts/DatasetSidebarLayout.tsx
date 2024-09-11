import { JSX } from "solid-js";

interface Props {
  children?: JSX.Element;
}

export const DatasetLayout = (props: Props) => {
  return (
    <div class="grid max-h-full grow grid-cols-[200px_1fr] overflow-hidden bg-green-500">
      <div class="h-full bg-red-500">sidebar</div>
      <div class="overflow-scroll">{props.children}</div>
    </div>
  );
};
