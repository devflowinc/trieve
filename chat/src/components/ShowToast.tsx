import { Show, createEffect, createSignal, onCleanup } from "solid-js";
import { BsCheck2Circle } from 'solid-icons/bs'
import { BiRegularErrorCircle } from 'solid-icons/bi'
import { VsClose } from 'solid-icons/vs'

const ShowToast = () => {
  const [isVisible, setIsVisible] = createSignal(false);
  const [toastDetails, setToastDetails] = createSignal({ type: "", message: "" });
  let timeOutId: number;

  createEffect(() => {
    const showToastEvent = (event: any) => {
      setIsVisible(true);
      setToastDetails(event.detail);

      timeOutId = setTimeout(() => {
        setIsVisible(false);
      }, 5000);
    };

    window.addEventListener("show-toast", showToastEvent);
    onCleanup(() => {
      clearTimeout(timeOutId);
    });
  });

  function handleToast() {
    setIsVisible(false);
  }

  return (
    <Show when={isVisible()}>
      <div class={`flex items-center dark:bg-neutral-900 bg-gray-100 text-center z-100 justify-between space-x-4 w-auto fixed top-10 right-5 px-5 py-2 border-t-4 ${toastDetails().type === "success" ? "border-green-600" : "border-red-700"} shadow-lg`}>
        {toastDetails().type === "success" ? <BsCheck2Circle class="text-green-600" size={25}/> : <BiRegularErrorCircle class="text-red-700" size={20}/>}
        <p class="text-md">{toastDetails().message}</p>
        <VsClose cursor="pointer" onClick={handleToast} size={25}/>
      </div>
    </Show>
  );
}

export default ShowToast;
