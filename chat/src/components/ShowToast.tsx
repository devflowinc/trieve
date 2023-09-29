import { For, createEffect, createSignal, onCleanup } from "solid-js";
import { BsCheck2Circle } from "solid-icons/bs";
import { BiRegularErrorCircle } from "solid-icons/bi";
import { VsClose } from "solid-icons/vs";

export interface ToastDetail {
  type: string;
  message: string;
}

export interface ToastEvent {
  detail: ToastDetail;
}

const ShowToast = () => {
  const [toastDetails, setToastDetails] = createSignal<ToastDetail[]>([]);

  createEffect(() => {
    let timeOutId: number;

    const showToastEvent = (event: Event) => {
      const toastEvent = event as unknown as ToastEvent;
      setToastDetails((prev) => prev.concat(toastEvent.detail));

      timeOutId = setTimeout(() => {
        setToastDetails((prev) =>
          prev.filter(
            (prevToastDetail) => prevToastDetail !== toastEvent.detail,
          ),
        );
      }, 5000);
    };

    window.addEventListener("show-toast", showToastEvent);

    onCleanup(() => {
      clearTimeout(timeOutId);
      window.removeEventListener("show-toast", showToastEvent);
    });
  });

  return (
    <div class="z-100 fixed right-5 top-10 flex flex-col space-y-2 rounded">
      <For each={toastDetails()}>
        {(toastDetail) => (
          <div
            class={`flex w-auto items-center justify-between space-x-4 rounded border-t-4 bg-gray-100 px-5 py-2 text-center dark:bg-neutral-900 ${
              toastDetail.type === "success"
                ? "border-green-600"
                : "border-red-700"
            } shadow-lg`}
          >
            {toastDetail.type === "success" ? (
              <BsCheck2Circle class="text-green-600" size={25} />
            ) : (
              <BiRegularErrorCircle class="text-red-700" size={20} />
            )}
            <p class="text-md">{toastDetail.message}</p>
            <VsClose
              cursor="pointer"
              onClick={() => {
                setToastDetails((prev) =>
                  prev.filter(
                    (prevToastDetail) => prevToastDetail !== toastDetail,
                  ),
                );
              }}
              size={25}
            />
          </div>
        )}
      </For>
    </div>
  );
};

export default ShowToast;
