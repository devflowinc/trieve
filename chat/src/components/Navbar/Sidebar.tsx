/* eslint-disable prettier/prettier */
import {
  BiRegularBrain,
  BiRegularChat,
  BiRegularCheck,
  BiRegularLogOut,
  BiRegularPlus,
  BiRegularTrash,
  BiRegularX,
} from "solid-icons/bi";
import {
  Accessor,
  createEffect,
  createSignal,
  For,
  Setter,
  Show,
} from "solid-js";
import type { Topic } from "~/types/topics";
import { FiSettings } from "solid-icons/fi";
import { FullScreenModal } from "../Atoms/FullScreenModal";
import { OnScreenThemeModeController } from "../Atoms/OnScreenThemeModeController";
import { AiFillGithub } from "solid-icons/ai";
import { TbMinusVertical } from "solid-icons/tb";

export interface SidebarProps {
  topics: Accessor<Topic[]>;
  refetchTopics: () => Promise<Topic[]>;
  setIsCreatingTopic: (value: boolean) => boolean;
  currentTopic: Accessor<Topic | undefined>;
  setCurrentTopic: (topic: Topic | undefined) => void;
  setSideBarOpen: Setter<boolean>;
  setIsCreatingNormalTopic: Setter<boolean>;
}

export const Sidebar = (props: SidebarProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;
  const dataset = import.meta.env.VITE_DATASET as unknown as string;
  const showGithubStars = import.meta.env
    .VITE_SHOW_GITHUB_STARS as unknown as string;

  const [editingIndex, setEditingIndex] = createSignal(-1);
  const [editingTopic, setEditingTopic] = createSignal("");
  const [settingsModalOpen, setSettingsModalOpen] = createSignal(false);
  const [starCount, setStarCount] = createSignal(0);

  const submitEditText = async () => {
    const topics = props.topics();
    const topic = topics[editingIndex()];

    const res = await fetch(`${apiHost}/topic`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      body: JSON.stringify({
        topic_id: topic.id,
        side: topic.side,
        resolution: editingTopic(),
      }),
    });

    if (!res.ok) {
      console.log("Error changing topic name (need toast)");
      return;
    }

    setEditingIndex(-1);
    void props.refetchTopics();
  };

  const deleteSelected = async () => {
    const res = await fetch(`${apiHost}/topic`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      body: JSON.stringify({
        topic_id: props.currentTopic()?.id,
      }),
    });

    if (res.ok) {
      props.setCurrentTopic(undefined);
      void props.refetchTopics();
    }
  };

  const logout = () => {
    void fetch(`${apiHost}/auth`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
    }).then((response) => {
      if (!response.ok) {
        return;
      }
      window.location.href = "/auth/login";
    });
  };

  createEffect(() => {
    try {
      void fetch(`https://api.github.com/repos/arguflow/arguflow`, {
        headers: {
          Accept: "application/vnd.github+json",
        },
      }).then((response) => {
        if (!response.ok) {
          return;
        }
        void response.json().then((data) => {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          setStarCount(data.stargazers_count);
        });
      });
    } catch (e) {
      console.error(e);
    }
  });

  return (
    <div class="absolute z-50 flex h-screen w-screen flex-row dark:text-gray-50 lg:relative lg:w-full">
      <div class="flex h-full w-2/3 flex-col bg-neutral-50 dark:bg-neutral-800 lg:w-full">
        <div class="flex w-full flex-col space-y-2 px-2 py-2 ">
          <button
            onClick={() => {
              props.setIsCreatingNormalTopic(false);
              props.setIsCreatingTopic(true);
              props.setCurrentTopic(undefined);
              props.setSideBarOpen(false);
            }}
            class="flex w-full flex-row items-center rounded-md border border-neutral-500 px-3 py-1 hover:bg-neutral-200  dark:border-neutral-400 dark:hover:bg-neutral-700"
          >
            <div class="flex flex-row items-center space-x-2">
              <span class="text-xl">
                <BiRegularPlus class="fill-current" />
              </span>
              <span>Retrieval Augmented Chat</span>
            </div>
          </button>
          <button
            onClick={() => {
              props.setIsCreatingTopic(false);
              props.setIsCreatingNormalTopic(true);
              props.setCurrentTopic(undefined);
              props.setSideBarOpen(false);
            }}
            class="flex w-full flex-row items-center rounded-md border border-neutral-500 px-3 py-1 hover:bg-neutral-200  dark:border-neutral-400 dark:hover:bg-neutral-700"
          >
            <div class="flex flex-row items-center space-x-2">
              <span class="text-xl">
                <BiRegularPlus class="fill-current" />
              </span>
              <span>Normal Chat</span>
            </div>
          </button>
        </div>
        <div class="flex w-full flex-col space-y-2 overflow-y-auto overflow-x-hidden px-2 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md dark:scrollbar-track-neutral-800 dark:scrollbar-thumb-neutral-600">
          <For each={props.topics()}>
            {(topic, index) => (
              <button
                classList={{
                  "flex items-center space-x-4 py-2 w-full rounded-md": true,
                  "bg-neutral-200 dark:bg-neutral-700":
                    props.currentTopic()?.id === topic.id,
                }}
                onClick={() => {
                  const topics = props.topics();
                  const topic = topics[index()];

                  props.setCurrentTopic(topic);
                  props.setIsCreatingTopic(false);
                  props.setIsCreatingNormalTopic(false);
                  props.setSideBarOpen(false);
                }}
              >
                {editingIndex() === index() && (
                  <div class="flex flex-1 items-center justify-between px-2">
                    <input
                      value={editingTopic()}
                      onInput={(e) => {
                        setEditingTopic(e.currentTarget.value);
                      }}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") {
                          void submitEditText();
                        }
                      }}
                      class="w-full rounded-md bg-neutral-50 px-2 py-1 dark:bg-neutral-800"
                    />

                    <div class="flex flex-row space-x-1 pl-2 text-2xl ">
                      <button
                        onClick={() => {
                          void submitEditText();
                        }}
                        class="hover:text-green-500"
                      >
                        <BiRegularCheck />
                      </button>
                      <button
                        onClick={(e) => {
                          e.preventDefault();
                          setEditingIndex(-1);
                        }}
                        class="hover:text-red-500"
                      >
                        <BiRegularX />
                      </button>
                    </div>
                  </div>
                )}
                {editingIndex() !== index() && (
                  <div class="flex flex-1 items-center px-3">
                    <Show when={topic.normal_chat}>
                      <BiRegularChat class="mr-2 fill-current" />
                    </Show>
                    <Show when={!topic.normal_chat}>
                      <BiRegularBrain class="mr-2 fill-current" />
                    </Show>
                    <p class="line-clamp-1 break-all">{topic.resolution}</p>
                    <div class="flex-1" />
                    <div class="flex flex-row items-center space-x-2">
                      {props.currentTopic() == topic && (
                        <div class="text-lg hover:text-purple-500">
                          <BiRegularTrash
                            class="fill-current"
                            onClick={() => {
                              void deleteSelected();
                            }}
                          />
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </button>
            )}
          </For>
        </div>
        <div class="flex-1 " />
        <div class="flex w-full flex-col space-y-1 border-t px-2 py-2 dark:border-neutral-400">
          <button
            class="flex w-full items-center space-x-4  rounded-md px-3 py-2 hover:bg-neutral-200   dark:hover:bg-neutral-700"
            onClick={logout}
          >
            <BiRegularLogOut class="h-6 w-6 fill-current" />
            <div>Logout</div>
          </button>
          <button
            class="flex w-full items-center space-x-4 rounded-md px-3 py-2 hover:bg-neutral-200 dark:hover:bg-neutral-700"
            onClick={() => setSettingsModalOpen(true)}
          >
            <FiSettings class="h-6 w-6" />
            <div>Settings</div>
          </button>
          <Show when={showGithubStars !== "off"}>
            <a
              href="https://github.com/arguflow/arguflow"
              class="flex w-full items-center space-x-4 rounded-md px-3 py-2 hover:bg-neutral-200 dark:hover:bg-neutral-700"
            >
              <AiFillGithub class="h-6 w-6 fill-current" />
              <div class="flex">
                <p>STAR US</p>
                <TbMinusVertical class="h-6 w-6" />
                <p>{starCount()}</p>
              </div>
            </a>
          </Show>
          <a
            href="https://github.com/arguflow/arguflow"
            class="flex items-center space-x-1 px-3 py-2"
          >
            <img src="/logo_512.png" class="h-7 w-7" />
            <div>
              <div class="mb-[-4px] w-full text-end align-bottom text-xs leading-3 text-turquoise">
                {dataset}
              </div>
              <div class="align-top text-lg">
                <span>Arguflow</span>
                <span class="text-magenta">Chat</span>\
              </div>
            </div>
          </a>
        </div>
      </div>
      <button
        class="w-1/3 flex-col bg-gray-500/5 backdrop-blur-[3px] lg:hidden"
        onClick={(e) => {
          e.preventDefault();
          props.setSideBarOpen(false);
        }}
      >
        <div class="ml-4 text-3xl">
          <BiRegularX />
        </div>
      </button>

      <Show when={settingsModalOpen()}>
        <FullScreenModal
          isOpen={settingsModalOpen}
          setIsOpen={setSettingsModalOpen}
        >
          <div class="min-w-[250px] sm:min-w-[300px]">
            <div class="mb-4 text-xl font-bold">Settings</div>
            <div class="mb-6 flex flex-col space-y-2">
              <div class="flex w-full items-center justify-between space-x-4">
                <div>Theme:</div>
                <OnScreenThemeModeController />
              </div>
            </div>
          </div>
        </FullScreenModal>
      </Show>
    </div>
  );
};
