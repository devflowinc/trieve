import { Show, createEffect, createSignal, useContext } from "solid-js";
import MainLayout from "../components/Layouts/MainLayout";
import { Navbar } from "../components/Navbar/Navbar";
import { Sidebar } from "../components/Navbar/Sidebar";
import { UserContext } from "../components/contexts/UserContext";
import { Topic } from "../utils/apiTypes";

export const Chat = () => {
  const userContext = useContext(UserContext);
  const [selectedTopic, setSelectedTopic] = createSignal<Topic | undefined>(
    undefined,
  );
  const [sidebarOpen, setSideBarOpen] = createSignal<boolean>(true);
  const [isCreatingTopic, setIsCreatingTopic] = createSignal<boolean>(true);
  const [topics, setTopics] = createSignal<Topic[]>([]);
  const [loadingNewTopic, setLoadingNewTopic] = createSignal<boolean>(false);
  const [selectedNewTopic, setSelectedNewTopic] = createSignal<boolean>(false);

  const refetchTopics = async (): Promise<Topic[]> => {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const api_host: string = import.meta.env.VITE_API_HOST;

    const dataset = userContext.currentDataset?.();
    if (!dataset) return [];
    const response = await fetch(
      `${api_host}/topic/owner/${userContext.user?.()?.id ?? ""}`,
      {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          "TR-Dataset": dataset.dataset.id,
        },
        credentials: "include",
      },
    );

    setIsCreatingTopic(true);
    setSelectedNewTopic(true);

    if (!response.ok) return [];
    const data: unknown = await response.json();
    if (data !== null && typeof data === "object" && Array.isArray(data)) {
      const topics = data as Topic[];
      setTopics(topics);
      return topics;
    }
    return [];
  };

  createEffect(() => {
    void refetchTopics();
  });

  createEffect(() => {
    if (selectedTopic()) {
      const updatedTopic = topics().find(
        (topic) => topic.id === selectedTopic()?.id,
      );
      if (updatedTopic && updatedTopic.name !== selectedTopic()?.name) {
        setSelectedTopic(updatedTopic);
      }
    }
  });

  return (
    <div class="relative flex h-screen flex-row bg-zinc-100 dark:bg-zinc-900">
      <div class="hidden w-1/4 overflow-x-hidden lg:block">
        <Sidebar
          currentTopic={selectedTopic}
          refetchTopics={refetchTopics}
          setCurrentTopic={setSelectedTopic}
          topics={topics}
          setIsCreatingTopic={setIsCreatingTopic}
          setSideBarOpen={setSideBarOpen}
          setSelectedNewTopic={setSelectedNewTopic}
        />
      </div>
      <div class="lg:hidden">
        <Show when={sidebarOpen()}>
          <Sidebar
            currentTopic={selectedTopic}
            refetchTopics={refetchTopics}
            setCurrentTopic={(topic: Topic | undefined) => {
              setIsCreatingTopic(false);
              setSelectedTopic(topic);
            }}
            topics={topics}
            setIsCreatingTopic={setIsCreatingTopic}
            setSelectedNewTopic={setSelectedNewTopic}
            setSideBarOpen={setSideBarOpen}
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
          loadingNewTopic={loadingNewTopic()}
          setSelectedNewTopic={setSelectedNewTopic}
          refetchTopics={refetchTopics}
          setSelectedTopic={setSelectedTopic}
          topics={topics}
        />
        <MainLayout
          setTopics={setTopics}
          setSelectedTopic={setSelectedTopic}
          selectedTopic={selectedTopic()}
          isCreatingTopic={isCreatingTopic()}
          setLoadingNewTopic={setLoadingNewTopic}
          selectedNewTopic={selectedNewTopic}
          setSelectedNewTopic={setSelectedNewTopic}
        />
      </div>
    </div>
  );
};

export default Chat;
