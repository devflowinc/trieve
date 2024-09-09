import { JSX, Show, splitProps } from "solid-js";

interface ChartCardProps extends JSX.HTMLAttributes<HTMLDivElement> {
  title?: string;
  subtitle?: string;
  width?: number;
  children: JSX.Element;
  controller?: JSX.Element;
}

export const ChartCard = (props: ChartCardProps) => {
  const [classStuff, others] = splitProps(props, ["class"]);
  return (
    <div
      {...others}
      style={{
        "grid-column": `span ${props.width}`,
      }}
      class={`shadow-xs rounded-lg border border-neutral-300 bg-white p-4 ${classStuff.class}`}
    >
      <div class="flex items-center justify-between mb-4">
        <div>
          <Show when={props.title}>
            {(title) => <div class="text-lg leading-none">{title()}</div>}
          </Show>
          <Show when={props.subtitle}>
            {(subtitle) => (
              <div class="text-sm leading-none text-neutral-600">
                {subtitle()}
              </div>
            )}
          </Show>
        </div>
        <Show when={props.controller}>{(controller) => controller()}</Show>
      </div>
      {props.children}
    </div>
  );
};
