import { JSX } from "solid-js/jsx-runtime";

interface SelectProps extends JSX.InputHTMLAttributes<HTMLSelectElement> {}

export const Select = (props: SelectProps) => {
  return <select class="rounded border border-neutral-300 p-1" {...props} />;
};
