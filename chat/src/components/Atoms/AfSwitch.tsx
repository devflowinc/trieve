import { createEffect, JSX } from "solid-js";
import { createFocusRing } from "@solid-aria/focus";
import { createSwitch } from "@solid-aria/switch";
import { createVisuallyHidden } from "@solid-aria/visually-hidden";

export interface AfSwitchProps {
  setIsOn: (value: boolean) => void;
  children?: JSX.Element;
}

export const AfSwitch = (props: AfSwitchProps) => {
  let ref: HTMLInputElement | undefined;

  const { inputProps, state } = createSwitch({}, () => ref);
  const { isFocusVisible, focusProps } = createFocusRing();
  const { visuallyHiddenProps } = createVisuallyHidden();

  createEffect(() => {
    props.setIsOn(state.isSelected());
  });

  return (
    <label style={{ display: "flex", "align-items": "center" }}>
      <div {...visuallyHiddenProps}>
        <input {...inputProps} {...focusProps} ref={ref} />
      </div>
      <svg
        width={40}
        height={24}
        aria-hidden="true"
        style={{ "margin-right": "4px" }}
      >
        <rect
          x={4}
          y={4}
          width={32}
          height={16}
          rx={8}
          fill={state.isSelected() ? "green" : "red"}
        />
        <circle cx={state.isSelected() ? 28 : 12} cy={12} r={5} fill="white" />
        {isFocusVisible() && (
          <rect
            x={1}
            y={1}
            width={38}
            height={22}
            rx={11}
            fill="none"
            stroke="orange"
            stroke-width={2}
          />
        )}
      </svg>
      {props.children && props.children}
    </label>
  );
};
