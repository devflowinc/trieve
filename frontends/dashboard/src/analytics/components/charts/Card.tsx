import { JSX, Show, splitProps } from "solid-js";

interface CardProps extends JSX.HTMLAttributes<HTMLDivElement> {
  title?: string;
  subtitle?: string;
  width?: number;
  children: JSX.Element;
  controller?: JSX.Element;
}

export const Card = (props: CardProps) => {
  const [classStuff, others] = splitProps(props, ["class"]);
  return (
    <div
      {...others}
      style={{
        "grid-column": `span ${props.width}`,
      }}
      class={`rounded-md border border-neutral-300 bg-white p-4 shadow-sm ${classStuff.class}`}
    >
      <div class="mb-4 flex items-center justify-between">
        <div>
          <Show when={props.title}>
            {(title) => (
              <div class="text-lg font-medium leading-none">{title()}</div>
            )}
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
