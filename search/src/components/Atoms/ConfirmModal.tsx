import { Accessor, Setter, Show } from "solid-js";
import { FullScreenModal } from "./FullScreenModal";
import { BiRegularXCircle } from "solid-icons/bi";
import { FiTrash } from "solid-icons/fi";

interface ConfirmModalProps {
  showConfirmModal: Accessor<boolean>;
  setShowConfirmModal: Setter<boolean>;
  onConfirm: Accessor<() => void>;
  checkMessage?: string;
  checked?: Accessor<boolean>;
  setChecked?: Setter<boolean>;
  message: string;
}

export const ConfirmModal = (props: ConfirmModalProps) => {
  return (
    <Show when={props.showConfirmModal()}>
      <FullScreenModal
        isOpen={props.showConfirmModal}
        setIsOpen={props.setShowConfirmModal}
      >
        <div class="min-w-[250px] sm:min-w-[300px]">
          <BiRegularXCircle class="mx-auto h-8 w-8 fill-current !text-red-500" />
          <div class="mb-6">
            <div class="text-center text-xl font-bold text-current dark:text-white">
              {props.message || "Are you sure you want to delete this?"}
            </div>
            <Show when={props.checkMessage}>
              {(knownCheckMessage) => (
                <div class="mt-2 flex items-center gap-2 text-center text-current dark:text-white">
                  <label>{knownCheckMessage()}:</label>
                  <input
                    class="h-4 w-4"
                    type="checkbox"
                    checked={props.checked?.()}
                    onChange={(e) => {
                      props.setChecked?.(e.target.checked);
                    }}
                  />
                </div>
              )}
            </Show>
          </div>
          <div class="mx-auto flex w-fit space-x-3">
            <button
              class="flex items-center space-x-2 rounded-md bg-magenta-500 p-2 text-white"
              onClick={() => {
                props.setShowConfirmModal(false);
                props.onConfirm()();
              }}
            >
              Delete
              <FiTrash class="h-5 w-5" />
            </button>
            <button
              class="flex space-x-2 rounded-md bg-neutral-500 p-2 text-white"
              onClick={() => props.setShowConfirmModal(false)}
            >
              Cancel
            </button>
          </div>
        </div>
      </FullScreenModal>
    </Show>
  );
};
