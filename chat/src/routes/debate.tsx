import { Transition } from "solid-headless";
import { Show, createEffect, createSignal } from "solid-js";
import { useSearchParams } from "solid-start";
import { NewTopicForm } from "~/components/Forms/NewTopicForm";
import Layout from "~/components/Layouts/MainLayout";
import { Navbar } from "~/components/Navbar/Navbar";
import { Sidebar } from "~/components/Navbar/Sidebar";
import { detectReferralToken, isTopic } from "~/types/actix-api";
import { Topic } from "~/types/topics";

export const debate = () => {
  const api_host: string = import.meta.env.VITE_API_HOST as unknown as string;

  const [searchParams] = useSearchParams();
  const [selectedTopic, setSelectedTopic] = createSignal<Topic | undefined>(
    undefined,
  );
  const [sidebarOpen, setSideBarOpen] = createSignal<boolean>(true);
  const [isCreatingTopic, setIsCreatingTopic] = createSignal<boolean>(false);
  const [isCreatingNormalTopic, setIsCreatingNormalTopic] =
    createSignal<boolean>(false);
  const [loadingTopic, setLoadingTopic] = createSignal<boolean>(false);
  const [isLogin, setIsLogin] = createSignal<boolean>(false);

  detectReferralToken(searchParams.t);

  createEffect(() => {
    void fetch(`${api_host}/auth`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
    }).then((response) => {
      setIsLogin(response.ok);
      if (
        !response.ok &&
        !(
          window.location.pathname.includes("/auth/") ||
          window.location.pathname === "/"
        )
      ) {
        window.location.href = "/auth/login";
        return;
      }
    });
  });

  const [topics, setTopics] = createSignal<Topic[]>([]);

  const refetchTopics = async (): Promise<Topic[]> => {
    const response = await fetch(`${api_host}/topic`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
    });
    if (!response.ok) return [];

    const data: unknown = await response.json();
    if (data !== null && typeof data === "object" && Array.isArray(data)) {
      const topics = data.filter((topic: unknown) => {
        return isTopic(topic);
      }) as Topic[];
      setTopics(topics);
      return topics;
    }

    return [];
  };

  createEffect(() => {
    void refetchTopics();
  });

  return (
    <Show when={isLogin()}>
      <div class="relative flex h-screen flex-row bg-zinc-100 dark:bg-zinc-900">
        <div class="hidden w-1/3 lg:block">
          <Sidebar
            currentTopic={selectedTopic}
            setCurrentTopic={setSelectedTopic}
            refetchTopics={refetchTopics}
            topics={topics}
            setIsCreatingTopic={setIsCreatingTopic}
            setSideBarOpen={setSideBarOpen}
            setIsCreatingNormalTopic={setIsCreatingNormalTopic}
          />
        </div>
        <div class="lg:hidden">
          <Show when={sidebarOpen()}>
            <Sidebar
              currentTopic={selectedTopic}
              setCurrentTopic={(topic: Topic | undefined) => {
                setIsCreatingTopic(false);
                setSelectedTopic(topic);
              }}
              refetchTopics={refetchTopics}
              topics={topics}
              setIsCreatingTopic={setIsCreatingTopic}
              setSideBarOpen={setSideBarOpen}
              setIsCreatingNormalTopic={setIsCreatingNormalTopic}
            />
          </Show>
        </div>
        <Show when={loadingTopic()}>
          <div class="flex w-full flex-col">
            <div class="flex w-full flex-col items-center justify-center">
              <img src="/cooking-crab.gif" class="aspect-square w-[128px]" />
            </div>
          </div>
        </Show>
        <Show
          when={
            !loadingTopic() &&
            !isCreatingTopic() &&
            selectedTopic() !== undefined
          }
        >
          <Transition
            class="flex w-full flex-col"
            show={
              !loadingTopic() &&
              !isCreatingTopic() &&
              selectedTopic() !== undefined
            }
            enter="transition-opacity duration-300"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="transition-opacity duration-300"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <div
              id="topic-layout"
              class="overflow-y-auto scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md dark:scrollbar-track-neutral-800 dark:scrollbar-thumb-neutral-600"
            >
              <Navbar
                selectedTopic={selectedTopic}
                setSideBarOpen={setSideBarOpen}
                setIsCreatingTopic={setIsCreatingTopic}
                setIsCreatingNormalTopic={setIsCreatingNormalTopic}
              />
              <Layout selectedTopic={selectedTopic} />
            </div>
          </Transition>
        </Show>
        <Show when={!loadingTopic() && (isCreatingTopic() || !selectedTopic())}>
          <Transition
            class="flex w-full flex-col space-y-16"
            show={!loadingTopic() && (isCreatingTopic() || !selectedTopic())}
            enter="transition-opacity duration-300"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="transition-opacity duration-300"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <Navbar
              selectedTopic={selectedTopic}
              setSideBarOpen={setSideBarOpen}
              setIsCreatingTopic={setIsCreatingTopic}
              setIsCreatingNormalTopic={setIsCreatingNormalTopic}
            />
            <NewTopicForm
              onSuccessfulTopicCreation={() => {
                setLoadingTopic(true);
                setIsCreatingTopic(false);
                setTimeout(() => {
                  void refetchTopics().then((topics_result) => {
                    setSelectedTopic(topics_result[0]);
                    setLoadingTopic(false);
                  });
                }, 500);
              }}
              setIsCreatingTopic={setIsCreatingTopic}
              selectedTopic={selectedTopic}
              setCurrentTopic={setSelectedTopic}
              topics={topics}
              isCreatingNormalTopic={isCreatingNormalTopic}
              setIsCreatingNormalTopic={setIsCreatingNormalTopic}
            />
          </Transition>
        </Show>
      </div>
    </Show>
  );
};

export default debate;
