import { JSX, ParentComponent, splitProps } from "solid-js";

interface ChartCardProps extends JSX.HTMLAttributes<HTMLDivElement> {
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
      class={`rounded-lg bg-white p-2 shadow-md ${classStuff.class}`}
    >
      {props.children}
    </div>
  );
};
