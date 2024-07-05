import {
  Dialog,
  DialogOverlay,
  DialogPanel,
  Transition,
  TransitionChild,
} from "solid-headless";
import type { Accessor, JSX, Setter } from "solid-js";
import { Portal } from "solid-js/web";

export interface FullScreenModalProps {
  children: JSX.Element;
  isOpen: Accessor<boolean>;
  setIsOpen: Setter<boolean> | ((open: boolean) => void);
}

export const FullScreenModal = (props: FullScreenModalProps) => {
  return (
    <Portal>
      <Transition appear show={props.isOpen()}>
        <Dialog
          isOpen={props.isOpen()}
          class="overflow-none z-100 fixed inset-0"
          onClose={() => {
            props.setIsOpen(false);
            localStorage.setItem("notFirstVisit", "true");
          }}
        >
          <div class="flex h-screen items-center justify-center px-4">
            <DialogOverlay class="fixed inset-0 bg-gray-500/5 backdrop-blur-[3px]" />
            <span class="inline-block h-screen align-middle" aria-hidden="true">
              &#8203;
            </span>
            <TransitionChild
              enter="ease-out duration-200"
              enterFrom="opacity-0"
              enterTo="opacity-100"
              leave="ease-in duration-200"
              leaveFrom="opacity-100"
              leaveTo="opacity-0"
            >
              <DialogPanel class="inline-block w-full transform overflow-hidden rounded-2xl bg-neutral-100 p-6 text-left align-middle shadow-md transition-all dark:bg-neutral-800">
                {props.children}
              </DialogPanel>
            </TransitionChild>
          </div>
        </Dialog>
      </Transition>
    </Portal>
  );
};
