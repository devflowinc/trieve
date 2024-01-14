import { Show, createEffect, createSignal } from "solid-js";
import { useSearchParams } from "solid-start";
import MainLayout from "~/components/Layouts/MainLayout";
import { Navbar } from "~/components/Navbar/Navbar";
import { Sidebar } from "~/components/Navbar/Sidebar";
import { detectReferralToken, isTopic } from "~/types/actix-api";
import { Topic } from "~/types/topics";
import {
  DatasetAndUsageDTO,
  isDatasetAndUsageDTO,
  isUserDTO,
  OrganizationDTO,
  UserDTO,
} from "~/utils/apiTypes";

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
  const [user, setUser] = createSignal<UserDTO | null>(null);
  const [currentOrganization, setCurrentOrganization] =
    createSignal<OrganizationDTO | null>(null);
  const [organizations, setOrganizations] = createSignal<OrganizationDTO[]>([]);

  const [datasetsAndUsages, setDatasetsAndUsages] = createSignal<
    DatasetAndUsageDTO[]
  >([]);
  const [currentDataset, setCurrentDataset] =
    createSignal<DatasetAndUsageDTO | null>(null);

  createEffect(() => {
    const u = user();
    if (u !== null && typeof u === "object") {
      setOrganizations(u.orgs);
      setCurrentOrganization(u.orgs[0]);
    }
  });

  createEffect(() => {
    const organization = currentOrganization();

    if (organization) {
      void fetch(`${apiHost}/dataset/organization/${organization.id}`, {
        method: "GET",
        credentials: "include",
        headers: {
          "AF-Organization": organization.id,
        },
      }).then((res) => {
        if (res.ok) {
          void res
            .json()
            .then((data) => {
              if (data && Array.isArray(data)) {
                if (data.length === 0) {
                  setDatasetsAndUsages([]);
                }
                if (data.length > 0 && data.every(isDatasetAndUsageDTO)) {
                  setCurrentDataset(data[0]);
                  setDatasetsAndUsages(data);
                }
              }
            })
            .catch((err) => {
              console.log(err);
            });
        }
      });
    }
  });

  detectReferralToken(searchParams.t);

  createEffect(() => {
    void fetch(`${apiHost}/auth/me`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
    }).then((response) => {
      setIsLogin(response.ok);

      response
        .json()
        .then((data) => {
          if (isUserDTO(data)) {
            setUser(data);
          }
        })
        .catch(() => {
          setUser(null);
        });
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
            currentOrganization={currentOrganization}
            setCurrentOrganization={setCurrentOrganization}
            organizations={organizations}
            currentDataset={currentDataset}
            setCurrentDataset={setCurrentDataset}
            datasetsAndUsages={datasetsAndUsages}
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
              currentOrganization={currentOrganization}
              setCurrentOrganization={setCurrentOrganization}
              organizations={organizations}
              currentDataset={currentDataset}
              setCurrentDataset={setCurrentDataset}
              datasetsAndUsages={datasetsAndUsages}
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
