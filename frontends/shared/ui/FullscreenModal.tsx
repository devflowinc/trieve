import { Accessor, Show } from "solid-js";
import { JSX } from "solid-js/jsx-runtime";
import { Portal } from "solid-js/web";
import {
  Dialog,
  DialogOverlay,
  DialogPanel,
  DialogTitle,
  Transition,
  TransitionChild,
} from "terracotta";
import { cn } from "../utils";
import { IoClose } from "solid-icons/io";

interface FullScreenModalProps {
  children: JSX.Element;
  show: Accessor<boolean>;
  title?: string;
  setShow: (show: boolean) => void;
  icon?: JSX.Element;
  class?: string;
}

export const FullScreenModal = (props: FullScreenModalProps) => {
  return (
    <Portal>
      <Transition
        class="fixed inset-0 z-10 overflow-y-auto"
        appear
        show={props.show()}
      >
        <Dialog
          isOpen
          class="fixed inset-0 z-10 overflow-y-auto"
          onClose={() => props.setShow(false)}
        >
          <div class="min-h-screen px-4 flex items-center justify-center">
            <TransitionChild
              enter="ease-out duration-300"
              enterFrom="opacity-0"
              enterTo="opacity-100"
              leave="ease-in duration-200"
              leaveFrom="opacity-100"
              leaveTo="opacity-0"
            >
              <DialogOverlay class="fixed inset-0 bg-neutral-900 bg-opacity-50" />
            </TransitionChild>

            {/* This element is to trick the browser into centering the modal contents. */}
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
              <DialogPanel
                class={cn(
                  "inline-block w-full max-w-[80vw] p-6 my-8 text-left border border-neutral-100 align-middle transition-all transform bg-white shadow-xl rounded max-h-[80vh] overflow-auto",
                  props.class
                )}
              >
                <div class="absolute right-0 top-0 hidden pr-4 pt-4 sm:flex items-center gap-4 ext-neutral-400 ">
                  <Show when={props.icon}>{props.icon}</Show>
                  <button
                    onClick={() => props.setShow(false)}
                    type="button"
                    class="rounded-md bg-white t focus:outline-none hover:text-gray-500"
                  >
                    <span class="sr-only">Close</span>
                    <IoClose class="w-6 h-6" />
                  </button>
                </div>
                <Show when={props.title}>
                  {(title) => (
                    <div class="flex items-center justify-between">
                      <DialogTitle
                        as="h3"
                        class="text-lg font-medium leading-6 text-neutral-900 mb-4 max-w-[80%] text-ellipsis truncate"
                      >
                        {title()}
                      </DialogTitle>
                    </div>
                  )}
                </Show>
                {props.children}
              </DialogPanel>
            </TransitionChild>
          </div>
        </Dialog>
      </Transition>
    </Portal>
  );
};
