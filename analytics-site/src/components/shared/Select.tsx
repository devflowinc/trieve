import { splitProps } from "solid-js";
import { JSX } from "solid-js/jsx-runtime";

interface SelectProps extends JSX.InputHTMLAttributes<HTMLSelectElement> {}

export const Select = (props: SelectProps) => {
  const [options] = splitProps(props, ["options"]);
  return <select class="rounded border border-neutral-300 p-1" {...props} />;
};
