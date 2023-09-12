import {
  Dialog,
  DialogOverlay,
  DialogPanel,
  Transition,
  TransitionChild,
} from "solid-headless";
import { Accessor, JSX, Setter } from "solid-js";

export interface FullScreenModalProps {
  children: JSX.Element;
  isOpen: Accessor<boolean>;
  setIsOpen: Setter<boolean>;
}

export const FullScreenModal = (props: FullScreenModalProps) => {
  return (
    <Transition appear show={props.isOpen()}>
      <Dialog
        isOpen={props.isOpen()}
        class="overflow-none fixed inset-0 z-10"
        onClose={() => props.setIsOpen(false)}
      >
        <div class="flex h-screen items-center justify-center px-4">
          <TransitionChild
            enter="ease-out duration-300"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="ease-in duration-200"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <DialogOverlay class="fixed inset-0 bg-gray-500/5 backdrop-blur-[3px]" />
          </TransitionChild>
          <span class="inline-block h-screen align-middle" aria-hidden="true">
            &#8203;
          </span>
          <TransitionChild
            enter="ease-out duration-300"
            enterFrom="opacity-0 scale-95"
            enterTo="opacity-100 scale-100"
            leave="ease-in duration-200"
            leaveFrom="opacity-100 scale-100"
            leaveTo="opacity-0 scale-95"
          >
            <DialogPanel class="my-8 inline-block w-full max-w-md transform overflow-hidden rounded-2xl bg-neutral-50 p-6 text-left align-middle shadow-md transition-all dark:bg-neutral-800">
              {props.children}
            </DialogPanel>
          </TransitionChild>
        </div>
      </Dialog>
    </Transition>
  );
};
