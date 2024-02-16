import { A } from "@solidjs/router";
import { Show, createSignal } from "solid-js";
import RegisterOrUserProfile from "../RegisterOrUserProfile";
import { useStore } from "@nanostores/solid";
import { clientConfig } from "../../stores/envsStore";
import { currentDataset } from "../../stores/datasetStore";

export const Navbar = () => {
  const $envs = useStore(clientConfig);
  const $datasetName = useStore(currentDataset)()?.dataset.name;
  const createChunkFeature = $envs().CREATE_CHUNK_FEATURE;
  const uploadDocumentFeature = $envs().DOCUMENT_UPLOAD_FEATURE;

  const [isOpen, setIsOpen] = createSignal(false);

  return (
    <nav class="mb-8 bg-white dark:bg-shark-800 dark:text-white">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div class="flex h-16 items-center justify-between">
          <div class="mx-auto flex h-[60px] w-full max-w-6xl items-center justify-between">
            <a class="flex items-center space-x-2" href="/">
              <img
                class="w-6 sm:w-12"
                src="https://cdn.trieve.ai/trieve-logo.png"
                alt="Logo"
              />
              <div class="hidden min-[450px]:block ">
                <div class="mb-[-4px] w-full text-end align-bottom leading-3 text-turquoise sm:mb-[-10px] sm:text-lg">
                  {$datasetName}
                </div>
                <div class="min-[380px]:text-xl sm:text-4xl">
                  <span>Trieve</span>
                  <span class="text-magenta">Search</span>
                </div>
              </div>
            </a>
            <div class="flex w-full items-center justify-end space-x-1 sm:space-x-4">
              <A
                href="/group"
                class="hidden text-center min-[420px]:text-lg min-[920px]:block"
              >
                Groups
              </A>
              <A
                href="/file"
                class="hidden text-center min-[420px]:text-lg min-[920px]:block"
              >
                Files
              </A>
              <Show when={createChunkFeature}>
                <A
                  href="/create"
                  class="hidden text-center min-[420px]:text-lg min-[920px]:block"
                >
                  Create Doc Chunk
                </A>
              </Show>
              <Show when={uploadDocumentFeature}>
                <A
                  href="/upload"
                  class="hidden text-center min-[420px]:text-lg min-[920px]:block"
                >
                  Upload Files
                </A>
              </Show>
              <a
                href="https://docs.trieve.ai"
                target="_blank"
                class="hidden min-[420px]:text-lg min-[920px]:block"
              >
                Docs
              </a>
              <div>
                <RegisterOrUserProfile />
              </div>
            </div>
          </div>
          <div class="-mr-2 flex md:hidden">
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
      </div>
      <div
        class={`${
          isOpen() ? "block" : "hidden"
        } bg-neutral-200 dark:bg-neutral-700 dark:text-white md:hidden`}
        id="mobile-menu"
      >
        <div class="space-y-1 px-2 pb-3 pt-2 sm:px-3">
          <Show when={createChunkFeature}>
            <a
              href="/create"
              class="block rounded-md bg-neutral-200 px-3 py-2 text-base font-medium hover:bg-neutral-300 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            >
              Create Doc Chunk
            </a>
          </Show>
          <Show when={uploadDocumentFeature}>
            <a
              href="/upload"
              class="block rounded-md bg-neutral-200 px-3 py-2 text-base font-medium hover:bg-neutral-300 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            >
              Upload Files
            </a>
          </Show>
          <a
            href="https://docs.trieve.ai"
            target="_blank"
            class="block rounded-md bg-neutral-200 px-3 py-2 text-base font-medium hover:bg-neutral-300 dark:bg-neutral-700 dark:hover:bg-neutral-800"
          >
            Docs
          </a>
        </div>
      </div>
    </nav>
  );
};
