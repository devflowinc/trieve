export const Spacer = (props: { h: number; withBorder: boolean }) => {
  return (
    <div
      classList={{
        "border-b border-b-neutral-300": props.withBorder,
      }}
      style={{ height: `${props.h}px` }}
    />
  );
};
