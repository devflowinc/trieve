import { Show, createEffect, createSignal } from "solid-js";
import { useSearchParams } from "solid-start";
import MainLayout from "~/components/Layouts/MainLayout";
import { Navbar } from "~/components/Navbar/Navbar";
import { Sidebar } from "~/components/Navbar/Sidebar";
import { detectReferralToken, isTopic } from "~/types/actix-api";
import { Topic } from "~/types/topics";

export const chat = () => {
  const apiHost: string = import.meta.env.VITE_API_HOST as unknown as string;

  const [searchParams] = useSearchParams();
  const [selectedTopic, setSelectedTopic] = createSignal<Topic | undefined>(
    undefined,
  );
  const [sidebarOpen, setSideBarOpen] = createSignal<boolean>(true);
  const [isCreatingTopic, setIsCreatingTopic] = createSignal<boolean>(true);
  const [isCreatingNormalTopic, setIsCreatingNormalTopic] =
    createSignal<boolean>(false);
  const [topics, setTopics] = createSignal<Topic[]>([]);
  const [isLogin, setIsLogin] = createSignal<boolean>(false);

  detectReferralToken(searchParams.t);

  createEffect(() => {
    void fetch(`${apiHost}/auth`, {
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

  const refetchTopics = async (): Promise<Topic[]> => {
    const response = await fetch(`${apiHost}/topic`, {
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
        <div class="hidden w-1/4 overflow-x-hidden lg:block">
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
        <div
          id="topic-layout"
          class="w-full overflow-y-auto scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md dark:scrollbar-track-neutral-800 dark:scrollbar-thumb-neutral-600"
        >
          <Navbar
            selectedTopic={selectedTopic}
            setSideBarOpen={setSideBarOpen}
            isCreatingTopic={isCreatingTopic}
            setIsCreatingTopic={setIsCreatingTopic}
            isCreatingNormalTopic={isCreatingNormalTopic}
            setIsCreatingNormalTopic={setIsCreatingNormalTopic}
          />
          <MainLayout
            setTopics={setTopics}
            setSelectedTopic={setSelectedTopic}
            isCreatingNormalTopic={isCreatingNormalTopic}
            selectedTopic={selectedTopic}
          />
        </div>
      </div>
    </Show>
  );
};

export default chat;
