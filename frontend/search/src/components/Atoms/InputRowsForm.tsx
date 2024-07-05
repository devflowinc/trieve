import { Accessor, For, JSXElement } from "solid-js";

export interface InputGroup {
  label: string;
  inputValue?: Accessor<string>;
  setInputValue?: (value: string) => void;
  type?: "input" | "textarea";
  component?: JSXElement;
}

export interface InputRowsFormProps {
  createButtonText: string;
  onCreate: () => void;
  onCancel: () => void;
  inputGroups: InputGroup[];
  borderColor?: string;
  buttonClass?: string;
}

const InputRowsForm = (props: InputRowsFormProps) => {
  return (
    <div class="w-full rounded px-2 py-2 dark:text-white">
      <div class="flex flex-col space-y-4">
        <div class="flex flex-col space-y-1">
          <For each={props.inputGroups}>
            {(inputGroup: InputGroup) => (
              <div class="w-full">
                <div>{inputGroup.label}:</div>
                {((!inputGroup.component && !inputGroup.type) ||
                  inputGroup.type == "input") && (
                  <input
                    class="w-full rounded border border-black px-2 focus:font-normal focus:blur-none dark:border-white dark:bg-neutral-500/80 dark:text-white"
                    type="text"
                    onInput={(e) =>
                      inputGroup.setInputValue?.(e.currentTarget.value)
                    }
                    value={inputGroup.inputValue?.()}
                  />
                )}
                {!inputGroup.component && inputGroup.type == "textarea" && (
                  <textarea
                    class="focusl:font-normal w-full rounded border border-black px-2 text-black focus:blur-none dark:border-white dark:bg-neutral-500/80 dark:text-white"
                    onInput={(e) =>
                      inputGroup.setInputValue?.(e.currentTarget.value)
                    }
                    value={inputGroup.inputValue?.()}
                  />
                )}
                {inputGroup.component && inputGroup.component}
              </div>
            )}
          </For>
        </div>
        <div class="mt-2 flex w-full justify-end space-x-2">
          <button
            class={`${
              props.buttonClass ?? ""
            } w-fit rounded-full border border-red-500 bg-transparent px-2 text-red-500`}
            onClick={() => props.onCancel()}
          >
            Cancel
          </button>
          <button
            class={`${
              props.buttonClass ?? ""
            } w-fit rounded-full border border-green-500 bg-transparent px-2 text-green-500`}
            onClick={() => props.onCreate()}
          >
            {props.createButtonText}
          </button>
        </div>
      </div>
    </div>
  );
};

export default InputRowsForm;
