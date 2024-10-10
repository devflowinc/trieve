import { Show } from "solid-js";

export type ValidateFn<T extends Record<string, unknown>> = (value: T) => {
  errors: ValidateErrors<T>;
  valid: boolean;
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type ValidateErrors<T extends Record<string, any>> = {
  [key in keyof T]: NonNullable<T[key]> extends Record<string, unknown>
    ? ReturnType<ValidateFn<NonNullable<T[key]>>>["errors"]
    : string | undefined;
};

export const ErrorMsg = (props: { error: string | null | undefined }) => {
  return (
    <Show when={props.error}>
      <div class="text-sm text-red-500">{props.error}</div>
    </Show>
  );
};
