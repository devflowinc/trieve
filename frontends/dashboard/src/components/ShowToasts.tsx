import { For, Show, createEffect, createSignal, onCleanup } from "solid-js";
import { BsCheck2Circle } from "solid-icons/bs";
import { BiRegularErrorCircle } from "solid-icons/bi";
import { VsClose } from "solid-icons/vs";

export interface ToastDetail {
  type: "success" | "error" | "info";
  title: string;
  message?: string;
  timeout?: number;
}

export interface ToastEvent {
  detail: ToastDetail;
}

export const createToast = ({ type, message, title, timeout }: ToastDetail) => {
  window.dispatchEvent(
    new CustomEvent("show-toast", {
      detail: {
        type,
        title,
        message,
        timeout,
      },
    }),
  );
};

const ShowToasts = () => {
  const [toastDetails, setToastDetails] = createSignal<ToastDetail[]>([]);

  createEffect(() => {
    let timeOutId: NodeJS.Timeout;

    const showToastEvent = (event: Event) => {
      const toastEvent = event as unknown as ToastEvent;
      setToastDetails((prev) => {
        if (
          prev.find(
            (prevToastDetail) =>
              prevToastDetail.message === toastEvent.detail.message,
          )
        ) {
          return prev;
        }
        return prev.concat(toastEvent.detail);
      });

      const timeoutMs = toastEvent.detail.timeout ?? 1500;

      timeOutId = setTimeout(() => {
        setToastDetails((prev) =>
          prev.filter(
            (prevToastDetail) => prevToastDetail !== toastEvent.detail,
          ),
        );
      }, timeoutMs);
    };

    window.addEventListener("show-toast", showToastEvent);

    onCleanup(() => {
      clearTimeout(timeOutId);
      window.removeEventListener("show-toast", showToastEvent);
    });
  });

  return (
    <div class="fixed bottom-5 left-5 z-50 flex flex-col space-y-2 rounded">
      <For each={toastDetails()}>
        {(toastDetail) => (
          <div class="pointer-events-auto min-w-full max-w-sm overflow-hidden rounded-lg bg-white shadow-lg ring-1 ring-black ring-opacity-5">
            <div class="p-4">
              <div class="flex items-start">
                <div class="flex-shrink-0">
                  <Show when={toastDetail.type === "success"}>
                    <BsCheck2Circle class="h-5 w-5 text-green-600" />
                  </Show>
                  <Show when={toastDetail.type === "error"}>
                    <BiRegularErrorCircle class="h-5 w-5 text-red-700" />
                  </Show>
                  <Show when={toastDetail.type === "info"}>
                    <BiRegularErrorCircle class="h-5 w-5 text-blue-700" />
                  </Show>
                </div>
                <div class="ml-3 flex-1 pt-0.5">
                  <p class="text-sm font-medium text-gray-900">
                    {toastDetail.title}
                  </p>
                  <Show when={toastDetail.message}>
                    <p class="mt-1 text-sm text-gray-500">
                      {toastDetail.message}
                    </p>
                  </Show>
                </div>
                <div class="ml-4 flex flex-shrink-0">
                  <VsClose
                    class="h-5 w-5 text-gray-400"
                    cursor="pointer"
                    onClick={() => {
                      setToastDetails((prev) =>
                        prev.filter(
                          (prevToastDetail) => prevToastDetail !== toastDetail,
                        ),
                      );
                    }}
                  />
                </div>
              </div>
            </div>
          </div>
        )}
      </For>
    </div>
  );
};

export default ShowToasts;
