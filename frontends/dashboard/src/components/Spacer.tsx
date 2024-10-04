import { cn } from "shared/utils";

export const Spacer = (props: {
  h: number;
  withBorder?: boolean;
  class?: string;
}) => {
  return (
    <div
      class={cn(
        props.withBorder && "border-b border-b-neutral-300",
        props.class,
      )}
      style={{ height: `${props.h}px` }}
    />
  );
};
