/* eslint-disable prettier/prettier */
import {
  BiRegularChat,
  BiRegularCheck,
  BiRegularPlus,
  BiRegularTrash,
  BiRegularX,
} from "solid-icons/bi";
import { drewsName } from "shared";
import {
  Accessor,
  createEffect,
  createSignal,
  For,
  Setter,
  Show,
  useContext,
} from "solid-js";
import { FiLink, FiSettings } from "solid-icons/fi";
import { FullScreenModal } from "../Atoms/FullScreenModal";
import { OnScreenThemeModeController } from "../Atoms/OnScreenThemeModeController";
import { AiFillGithub } from "solid-icons/ai";
import { TbMinusVertical } from "solid-icons/tb";
import { DatasetSelectBox } from "../Atoms/DatasetSelectBox";
import { OrganizationSelectBox } from "../Atoms/OrganizationSelectBox";
import { UserContext } from "../contexts/UserContext";
import { Topic } from "../../utils/apiTypes";
import { createToast } from "../ShowToast";

export interface SidebarProps {
  topics: Accessor<Topic[]>;
  refetchTopics: () => Promise<Topic[]>;
  setIsCreatingTopic: (value: boolean) => boolean;
  currentTopic: Accessor<Topic | undefined>;
  setCurrentTopic: (topic: Topic | undefined) => void;
  setSideBarOpen: Setter<boolean>;
  setSelectedNewTopic: Setter<boolean>;
}

export const Sidebar = (props: SidebarProps) => {
  const dashboardUrl = import.meta.env.VITE_DASHBOARD_URL as string;
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const [editingIndex, setEditingIndex] = createSignal(-1);
  const [editingTopic, setEditingTopic] = createSignal("");
  const [settingsModalOpen, setSettingsModalOpen] = createSignal(false);
  const [starCount, setStarCount] = createSignal(0);

  const userContext = useContext(UserContext);

  const submitEditText = async () => {
    const topics = props.topics();
    const topic = topics[editingIndex()];

    const dataset = userContext.currentDataset?.();
    if (!dataset) return;

    const res = await fetch(`${apiHost}/topic`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": dataset.dataset.id,
      },
      credentials: "include",
      body: JSON.stringify({
        topic_id: topic.id,
        name: editingTopic(),
        owner_id: userContext.user?.()?.id,
      }),
    });

    if (!res.ok) {
      createToast({
        type: "error",
        message: "Error changing topic name",
      });
      return;
    }

    setEditingIndex(-1);
    void props.refetchTopics();
  };

  const deleteSelected = async () => {
    const dataset = userContext.currentDataset?.();
    if (!dataset) return;

    const res = await fetch(`${apiHost}/topic/${props.currentTopic()?.id}`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": dataset.dataset.id,
      },
      credentials: "include",
    });

    if (res.ok) {
      props.setCurrentTopic(undefined);
      void props.refetchTopics();
    } else {
      createToast({
        type: "error",
        message: "Error deleting topic",
      });
      return;
    }
  };

  createEffect(() => {
    try {
      void fetch(`https://api.github.com/repos/devflowinc/trieve`, {
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
      <div class="flex h-full w-2/3 flex-col border-r border-neutral-300 bg-neutral-200 dark:border-neutral-700 dark:bg-neutral-800 lg:w-full">
        <div class="flex w-full flex-col space-y-2 px-2 py-2">
          {drewsName}
          <button
            onClick={() => {
              props.setIsCreatingTopic(true);
              props.setCurrentTopic(undefined);
              props.setSelectedNewTopic(true);
              props.setSideBarOpen(false);
            }}
            disabled={userContext.user?.() === null}
            class="flex w-full flex-row items-center rounded-md border border-transparent px-3 py-1 hover:border-neutral-300 hover:bg-neutral-100 disabled:border-neutral-300 disabled:bg-neutral-200 disabled:text-neutral-400 hover:dark:border-neutral-700 dark:hover:bg-neutral-700 hover:dark:bg-neutral-700/50"
          >
            <div class="flex flex-row items-center space-x-2">
              <span class="text-xl">
                <BiRegularPlus class="fill-current" />
              </span>
              <span>RAG Playground</span>
            </div>
          </button>
        </div>
        <div class="flex w-full flex-col space-y-2 overflow-y-auto overflow-x-hidden px-2 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md dark:scrollbar-track-neutral-800 dark:scrollbar-thumb-neutral-600">
          <For each={props.topics()}>
            {(topic, index) => (
              <button
                classList={{
                  "flex items-center space-x-4 py-2 w-full rounded-md": true,
                  "bg-white border border-neutral-300 dark:border-neutral-600/70 text-black dark:text-white dark:bg-neutral-700/50":
                    props.currentTopic()?.id === topic.id,
                }}
                onClick={() => {
                  const topics = props.topics();
                  const topic = topics[index()];

                  props.setCurrentTopic(topic);
                  props.setSelectedNewTopic(true);
                  props.setIsCreatingTopic(false);
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

                    <div class="flex flex-row space-x-1 pl-2 text-2xl">
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
                    <BiRegularChat class="mr-2 fill-current" />
                    <p class="line-clamp-1 break-all">{topic.name}</p>
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
        <div class="flex-1" />
        <div class="flex w-full flex-col space-y-1 border-t px-2 py-2 text-base dark:border-neutral-600">
          <div class="ml-4 flex items-center space-x-2">
            <OrganizationSelectBox />
            <p class="text-2xl">/</p>
            <DatasetSelectBox />
          </div>
          <a
            href={dashboardUrl}
            class="flex w-full items-center space-x-4 rounded-md px-3 py-2 hover:bg-neutral-200 disabled:border-neutral-300 disabled:bg-neutral-200 disabled:text-neutral-400 dark:hover:bg-neutral-700"
          >
            <FiLink class="h-4 w-4" />
            <div>Dashboard</div>
          </a>
          <button
            disabled={userContext.user?.() === null}
            class="flex w-full items-center space-x-4 rounded-md px-3 py-2 hover:bg-neutral-200 disabled:border-neutral-300 disabled:bg-neutral-200 disabled:text-neutral-400 dark:hover:bg-neutral-700"
            onClick={() => setSettingsModalOpen(true)}
          >
            <FiSettings class="h-4 w-4" />
            <div>Settings</div>
          </button>
          <a
            href="https://github.com/devflowinc/trieve"
            class="flex w-full items-baseline gap-4 rounded-md px-3 py-2 hover:bg-neutral-200 dark:hover:bg-neutral-700"
          >
            <AiFillGithub class="h-4 w-4 fill-current" />
            <div class="flex items-center">
              <p>Star Us</p>
              <TbMinusVertical class="h-4 w-4" />
              <p>{starCount()}</p>
            </div>
          </a>
          <a
            href="https://github.com/devflowinc/trieve"
            class="m-auto flex items-center gap-1 px-3 py-2"
          >
            <img src="https://cdn.trieve.ai/trieve-logo.png" class="h-7 w-7" />
            <div>
              <div class="mb-[-4px] w-full text-end align-bottom text-xs leading-3 text-turquoise">
                {userContext.currentDataset?.()?.dataset.name}
              </div>
              <div class="align-top text-lg">
                <span>Trieve</span>
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
