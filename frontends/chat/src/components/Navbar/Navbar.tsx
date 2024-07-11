import {
  BiRegularMenuAltLeft,
  BiRegularPlus,
  BiRegularEdit,
  BiRegularCheck,
  BiRegularX,
} from "solid-icons/bi";
import {
  Setter,
  Switch,
  Match,
  createSignal,
  Show,
  useContext,
  createEffect,
} from "solid-js";
import { Topic } from "../../utils/apiTypes";
import { UserContext } from "../contexts/UserContext";

export interface NavbarProps {
  setSideBarOpen: Setter<boolean>;
  selectedTopic: () => Topic | undefined;
  isCreatingTopic: () => boolean;
  setIsCreatingTopic: Setter<boolean>;
  loadingNewTopic: boolean;
  setSelectedNewTopic: Setter<boolean>;
  refetchTopics: () => Promise<Topic[]>;
  setSelectedTopic: Setter<Topic | undefined>;
  topics: () => Topic[];
}

export const Navbar = (props: NavbarProps) => {
  const api_host: string = import.meta.env.VITE_API_HOST as string;
  const userContext = useContext(UserContext);

  const [editing, setEditing] = createSignal(false);
  const [editedContent, setEditedContent] = createSignal("");
  const [showCheckmarkIcon, setShowCheckmarkIcon] = createSignal(true);
  const [previousTopicId, setPreviousTopicId] = createSignal<
    string | undefined
  >(undefined);

  const editTitle = () => {
    setEditing(true);
    setEditedContent(props.selectedTopic()?.name ?? "");
  };

  const saveTitle = async () => {
    const dataset = userContext.currentDataset?.();
    const selectedTopic = props.selectedTopic();

    if (!dataset || !selectedTopic) {
      return;
    }

    if (editedContent().trim()) {
      await fetch(`${api_host}/topic`, {
        method: "PUT",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
          "TR-Dataset": dataset.dataset.id,
        },
        body: JSON.stringify({
          topic_id: selectedTopic.id,
          name: editedContent().trim(),
        }),
      })
        .then((res) => {
          if (!res.ok) {
            console.error("Error updating topic name");
            setEditing(false);
            return;
          }
          selectedTopic.name = editedContent().trim();
          props.setSelectedNewTopic(false);
          setEditing(false);

          props
            .refetchTopics()
            .then((topics) => {
              const updatedTopic = topics.find(
                (topic) => topic.id === selectedTopic.id,
              );
              if (updatedTopic) {
                props.setSelectedTopic(updatedTopic);
              }
            })
            .catch((err) => {
              console.error(
                "Cannot refetch topics after updating topic name:",
                err,
              );
            });
        })
        .catch((err) => {
          console.error("Error with updating topic name:", err);
        });
    }
  };

  const handleSaveTitle = () => {
    saveTitle().catch((err) => {
      console.error(err);
    });
  };

  const cancelEdit = () => {
    setEditing(false);
    setEditedContent(props.selectedTopic()?.name ?? "");
  };

  createEffect(() => {
    setShowCheckmarkIcon(editedContent().trim() !== "");
  });

  createEffect(() => {
    const selectedTopic = props.selectedTopic();
    if (selectedTopic?.id !== previousTopicId()) {
      setEditing(false);
      setEditedContent("");
      setPreviousTopicId(selectedTopic?.id);
    }
  });

  return (
    <div class="flex w-full items-center justify-between border-b border-neutral-300 bg-neutral-200/80 px-5 py-3 font-semibold text-neutral-800 dark:border-neutral-800 dark:bg-neutral-800/50 dark:text-white md:text-xl">
      <div class="lg:hidden">
        <BiRegularMenuAltLeft
          onClick={() => props.setSideBarOpen((prev) => !prev)}
          class="fill-current text-4xl"
        />
      </div>
      <Switch>
        <Match when={props.loadingNewTopic}>
          <div class="flex w-full items-center justify-center px-2 text-center text-base">
            <p>Loading...</p>
          </div>
        </Match>
        <Match when={!props.loadingNewTopic}>
          <div class="flex min-h-8 w-full items-center justify-center px-2 text-center text-base">
            <Show
              when={editing()}
              fallback={
                <div class="flex flex-row items-center justify-center">
                  <p class="mr-2">
                    {props.selectedTopic()?.name ?? "New RAG Chat"}
                  </p>
                  <Show when={props.selectedTopic()}>
                    <BiRegularEdit onClick={editTitle} />
                  </Show>
                </div>
              }
            >
              <div class="flex flex-row items-center justify-center gap-x-1.5">
                <input
                  type="text"
                  value={editedContent()}
                  maxlength="150"
                  onInput={(e) => setEditedContent(e.currentTarget.value)}
                  onKeyUp={(e) => {
                    if (e.key === "Enter") handleSaveTitle();
                  }}
                  class="rounded-md border border-neutral-300 px-2 text-sm"
                />
                <Show when={showCheckmarkIcon()}>
                  <BiRegularCheck
                    class="hover:text-green-500"
                    onClick={handleSaveTitle}
                  />
                </Show>
                <BiRegularX onClick={cancelEdit} class="hover:text-red-500" />
              </div>
            </Show>
          </div>
        </Match>
      </Switch>
      <div class="lg:hidden">
        <BiRegularPlus
          onClick={() => {
            props.setSideBarOpen(false);
            props.setIsCreatingTopic(true);
            props.setSelectedNewTopic(true);
            console.log("setting new topic");
          }}
          class="fill-current text-4xl"
        />
      </div>
    </div>
  );
};
