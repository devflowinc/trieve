import { Dialog, DialogOverlay, DialogPanel } from "terracotta";
import { Accessor, JSX, Setter, Show } from "solid-js";

export interface FullScreenModalProps {
  children: JSX.Element;
  isOpen: Accessor<boolean>;
  setIsOpen: Setter<boolean>;
}

export const FullScreenModal = (props: FullScreenModalProps) => {
  return (
    <Show when={props.isOpen()}>
      <Dialog
        isOpen={props.isOpen()}
        class="overflow-none fixed inset-0 z-10"
        onClose={() => props.setIsOpen(false)}
      >
        <div class="flex h-screen items-center justify-center px-4">
          <DialogOverlay class="fixed inset-0 bg-gray-500/50" />
          <span class="inline-block h-screen align-middle" aria-hidden="true">
            &#8203;
          </span>
          <DialogPanel class="my-8 inline-block w-full max-w-md transform overflow-hidden rounded bg-neutral-50 p-6 text-left align-middle shadow-md transition-all dark:bg-neutral-800">
            {props.children}
          </DialogPanel>
        </div>
      </Dialog>
    </Show>
  );
};
