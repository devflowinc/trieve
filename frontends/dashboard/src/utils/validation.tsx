import { Show } from "solid-js";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type ValidateFn<T extends Record<string, any>> = (value: T) => {
  errors: {
    [key in keyof T]: string | undefined;
  };
  valid: boolean;
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type ValidateErrors<T extends ValidateFn<any>> = ReturnType<T>["errors"];

export const ErrorMsg = (props: { error: string | null | undefined }) => {
  return (
    <Show when={props.error}>
      <div class="text-sm text-red-500">{props.error}</div>
    </Show>
  );
};
