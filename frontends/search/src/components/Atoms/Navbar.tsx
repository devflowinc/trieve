import { Show, createMemo, createSignal, useContext } from "solid-js";
import { DatasetAndUserContext } from "../Contexts/DatasetAndUserContext";
import { OrganizationSelectBox } from "../OrganizationSelectBox";
import { DatasetSelectBox } from "../DatasetSelectBox";

export const Navbar = () => {
  const dashboardUrl = import.meta.env.VITE_DASHBOARD_URL as string;

  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $currentUser = datasetAndUserContext.user;

  const currentDatasetId = createMemo(() => {
    return datasetAndUserContext.currentDataset?.()?.dataset.id;
  });

  const [isOpen, setIsOpen] = createSignal(false);

  return (
    <div class="mx-auto mb-8 w-full max-w-screen-2xl flex-col items-center px-4">
      <div class="flex h-16 items-center justify-between">
        <div class="flex h-[60px] w-full items-center justify-between">
          <div class="flex min-w-fit items-center space-x-2">
            <a
              href={`/?dataset=${datasetAndUserContext.currentDataset?.()
                ?.dataset.id}`}
            >
              <img
                class="w-6 sm:w-12"
                src="https://cdn.trieve.ai/trieve-logo.png"
                alt="Logo"
              />
            </a>
            <Show when={$currentUser?.()}>
              <div class="flex items-center space-x-2">
                <OrganizationSelectBox />
                <span class="text-2xl">/</span>
                <DatasetSelectBox />
              </div>
            </Show>
          </div>

          <div class="flex items-center justify-end space-x-1 sm:space-x-4">
            <a
              href={dashboardUrl}
              class="hidden text-center min-[420px]:text-lg min-[920px]:block"
            >
              Dashboard
            </a>
            <a
              href="https://docs.trieve.ai/api-reference"
              class="hidden text-center min-[420px]:text-lg min-[920px]:block"
            >
              API Docs
            </a>
            <a
              href={`/?dataset=${datasetAndUserContext.currentDataset?.()
                ?.dataset.id}`}
              class="hidden text-center min-[420px]:text-lg min-[920px]:block"
            >
              Search Chunks
            </a>
            <a
              href={`/group?dataset=${currentDatasetId()}`}
              class="hidden text-center min-[420px]:text-lg min-[920px]:block"
            >
              Groups
            </a>
            <a
              href={`/create?dataset=${currentDatasetId()}`}
              class="hidden text-center min-[420px]:text-lg min-[920px]:block"
            >
              Create Chunk
            </a>
            <a
              href={`/upload?dataset=${currentDatasetId()}`}
              class="hidden text-center min-[420px]:text-lg min-[920px]:block"
            >
              Upload Files
            </a>
          </div>
        </div>
        <div class="flex md:hidden">
          <button
            type="button"
            class="ml-2 inline-flex items-center justify-center rounded-md bg-neutral-200 p-2 focus:outline-none focus:ring-1 focus:ring-neutral-800 focus:ring-offset-1 dark:bg-neutral-700 dark:focus:ring-white"
            aria-controls="mobile-menu"
            aria-expanded={isOpen()}
            onClick={(e) => {
              e.preventDefault();
              setIsOpen(!isOpen());
            }}
          >
            <span class="sr-only">Open main menu</span>
            <svg
              class={`${isOpen() ? "hidden" : "block"} h-6 w-6`}
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 6h16M4 12h16M4 18h16"
              />
            </svg>
            <svg
              class={`${isOpen() ? "block" : "hidden"} h-6 w-6`}
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>
      </div>
      <Show when={isOpen()}>
        <div
          class="-mx-4 bg-neutral-200 dark:bg-neutral-700 dark:text-white"
          id="mobile-menu"
        >
          <div class="space-y-1 px-2 pb-3 pt-2">
            <a
              href={dashboardUrl}
              class="block rounded-md bg-neutral-200 py-2 text-base font-medium hover:bg-neutral-300 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            >
              Dashboard
            </a>
            <a
              href="https://docs.trieve.ai/api-reference"
              class="block rounded-md bg-neutral-200 py-2 text-base font-medium hover:bg-neutral-300 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            >
              API Docs
            </a>
            <a
              href={`/?dataset=${datasetAndUserContext.currentDataset?.()
                ?.dataset.id}`}
              class="block rounded-md bg-neutral-200 py-2 text-base font-medium hover:bg-neutral-300 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            >
              Search Chunks
            </a>
            <a
              href={`/group?dataset=${currentDatasetId()}`}
              class="block rounded-md bg-neutral-200 py-2 text-base font-medium hover:bg-neutral-300 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            >
              Groups
            </a>
            <a
              href={`/create?dataset=${currentDatasetId()}`}
              class="block rounded-md bg-neutral-200 py-2 text-base font-medium hover:bg-neutral-300 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            >
              Create Chunk
            </a>
            <a
              href={`/upload?dataset=${currentDatasetId()}`}
              class="block rounded-md bg-neutral-200 py-2 text-base font-medium hover:bg-neutral-300 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            >
              Upload Files
            </a>
          </div>
        </div>
      </Show>
    </div>
  );
};
