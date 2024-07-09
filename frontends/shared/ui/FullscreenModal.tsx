import { Accessor, Show } from "solid-js";
import { JSX } from "solid-js/jsx-runtime";
import {
  Dialog,
  DialogOverlay,
  DialogPanel,
  DialogTitle,
  Transition,
  TransitionChild,
} from "terracotta";

interface FullScreenModalProps {
  children: JSX.Element;
  show: Accessor<boolean>;
  title?: string;
  setShow: (show: boolean) => void;
  icon?: JSX.Element;
}

export const FullScreenModal = (props: FullScreenModalProps) => {
  return (
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
            <DialogPanel class="inline-block w-full max-w-md p-6 my-8 overflow-hidden text-left border border-neutral-100 align-middle transition-all transform bg-white shadow-xl rounded">
              <Show when={props.title}>
                {(title) => (
                    <div class="flex items-center justify-between">
                      <DialogTitle
                        as="h3"
                        class="text-lg font-medium leading-6 text-neutral-900"
                      >
                        {title()}
                      </DialogTitle>
                      <Show when={props.icon}> 
                        {props.icon}
                      </Show>
                  </div>
                )}
              </Show>
              {props.children}
            </DialogPanel>
          </TransitionChild>
        </div>
      </Dialog>
    </Transition>
  );
};
