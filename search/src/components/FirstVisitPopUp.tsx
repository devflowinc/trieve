import { Show, createSignal } from "solid-js";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import { VsChromeClose } from "solid-icons/vs";

export const FirstVisitPopUp = () => {
  const [isOpen, setIsOpen] = createSignal(true);
  const notFirstVisit = localStorage.getItem("notFirstVisit");

  return (
    <Show when={!notFirstVisit}>
      <FullScreenModal isOpen={isOpen} setIsOpen={setIsOpen}>
        <div class="flex-col items-center justify-between space-y-2">
          <VsChromeClose
            class="absolute right-4 top-4 h-6 w-6 cursor-pointer"
            onClick={() => {
              setIsOpen(false);
              localStorage.setItem("notFirstVisit", "true");
            }}
          />
          <img class="w-12" src="/logo_transparent.png" alt="Logo" />
          <p class="text-2xl font-bold">👋 Welcome to Arguflow Search!</p>
          <p class="text-md font-semibold">
            We recognize that its your first time here and encourage you to
            check out our{" "}
            <a
              class="border-none text-turquoise-500 underline outline-none ring-0 dark:text-acid-500"
              href="https://docs.arguflow.ai"
            >
              feature list
            </a>{" "}
            or watch our tutorial video to get started
          </p>
          <div class="pt-3">
            <iframe
              class="h-48 w-full rounded sm:h-96"
              src="https://www.youtube.com/embed/fcYem3u7Cvo"
              title="YouTube video player"
              allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
              allowfullscreen
            />
          </div>
        </div>
      </FullScreenModal>
    </Show>
  );
};
