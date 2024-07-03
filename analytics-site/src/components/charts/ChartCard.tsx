import { JSX, Show, splitProps } from "solid-js";

interface ChartCardProps extends JSX.HTMLAttributes<HTMLDivElement> {
  title?: string;
  width: number;
  children: JSX.Element;
}

export const ChartCard = (props: ChartCardProps) => {
  const [classStuff, others] = splitProps(props, ["class"]);
  return (
    <div
      {...others}
      style={{
        "grid-column": `span ${props.width}`,
      }}
      class={`shadow-xs rounded-lg border border-neutral-300 bg-white p-2 ${classStuff.class}`}
    >
      <Show when={props.title}>
        {(title) => <div class="mb-2 ml-2 text-lg">{title()}</div>}
      </Show>
      {props.children}
    </div>
  );
};
