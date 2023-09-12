import { Show, createSignal, Accessor, Setter } from "solid-js";
import { isActixApiDefaultError } from "~/types/actix-api";
import type { Topic } from "~/types/topics";

export interface NewTopicFormProps {
  onSuccessfulTopicCreation: () => void;
  setIsCreatingTopic: (value: boolean) => void;
  selectedTopic: () => Topic | undefined;
  setCurrentTopic: (topic: Topic | undefined) => void;
  topics: Accessor<Topic[]>;
  isCreatingNormalTopic: Accessor<boolean>;
  setIsCreatingNormalTopic: Setter<boolean>;
}

export const NewTopicForm = (props: NewTopicFormProps) => {
  const [topicName, setTopicName] = createSignal("");

  const [errorMessage, setErrorMessage] = createSignal("");

  const api_host: string = import.meta.env.VITE_API_HOST as unknown as string;

  const processResponse = (response: Response) => {
    if (!response.ok) {
      void response.json().then((data) => {
        if (isActixApiDefaultError(data)) {
          setErrorMessage(data.message);
        }
      });
      return;
    }
    props.onSuccessfulTopicCreation();
  };

  return (
    <div class="flex w-full flex-col px-10 align-middle text-neutral-900 dark:text-neutral-50">
      <form class="flex w-full flex-col space-y-4">
        <p class="w-full text-center text-2xl font-semibold">
          Create New{" "}
          {props.isCreatingNormalTopic() ? "Regular Chat" : "Debate Topic"}
        </p>
        <div class="text-center text-red-500">{errorMessage()}</div>
        <label for="topicName"> Topic Name</label>
        <input
          type="topicName"
          name="topicName"
          id="topicName"
          class="rounded border border-neutral-300 p-2 text-neutral-900 dark:border-neutral-700"
          value={topicName()}
          onInput={(e) => setTopicName(e.currentTarget.value)}
        />
        <div class="flex w-full space-x-2">
          <button
            type="submit"
            class="w-full rounded bg-neutral-200 p-2  dark:bg-neutral-700"
            onClick={(e) => {
              e.preventDefault();

              const isNormalTopic = props.isCreatingNormalTopic();

              let body: object = {
                resolution: topicName(),
              };

              if (isNormalTopic) {
                body = {
                  resolution: topicName(),
                  normal_chat: true,
                };
              }

              void fetch(`${api_host}/topic`, {
                method: "POST",
                headers: {
                  "Content-Type": "application/json",
                },
                credentials: "include",
                body: JSON.stringify(body),
              }).then(processResponse);
            }}
          >
            Submit
          </button>
          <Show when={props.selectedTopic()}>
            <button
              class="w-full rounded bg-neutral-200 p-2  dark:bg-neutral-700"
              onClick={() => {
                props.setIsCreatingTopic(false);
              }}
            >
              Cancel
            </button>
          </Show>
        </div>
      </form>
    </div>
  );
};
