import { makePersisted } from "@solid-primitives/storage";
import { IconTypes } from "solid-icons";
import {
  BsHandThumbsDown,
  BsHandThumbsDownFill,
  BsHandThumbsUp,
  BsHandThumbsUpFill,
} from "solid-icons/bs";
import { Show, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { UserContext } from "../contexts/UserContext";

interface MessageVotingProps {
  queryId: string;
}

type MessageVoteStore = {
  [key: string]: number;
};

const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

// Create store exepcted to be directly destructured, this is fine
// eslint-disable-next-line @typescript-eslint/no-unused-vars, solid/reactivity
const [store, setStore] = makePersisted(createStore<MessageVoteStore>({}), {
  name: "vote-cache",
});

export const MessageVoting = (props: MessageVotingProps) => {
  const userContext = useContext(UserContext);

  const updateVote = async (queryId: string, vote: number) => {
    const prevVote = store[queryId];
    setStore(queryId, vote);
    const dataset = userContext.currentDataset?.();
    if (dataset?.dataset.id) {
      const response = await fetch(apiHost + "/analytics/rag", {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
          "TR-Dataset": dataset.dataset.id,
        },
        credentials: "include",
        body: JSON.stringify({
          query_id: queryId,
          rating: vote,
        }),
      });

      if (response.status !== 204) {
        console.error("Failed to update vote", response);
        // Rollback the vote
        setStore(queryId, prevVote);
      }
    }
  };

  const VoteIcon = (props: {
    icon: IconTypes;
    onClick: (e: MouseEvent) => void;
  }) => {
    return <div onClick={(e) => props.onClick(e)}>{props.icon({})}</div>;
  };

  return (
    <>
      <div class="my-2 mb-2 w-full border-b border-b-neutral-400 dark:border-b-neutral-500" />
      <div class="flex flex-col gap-2 pt-1">
        <Show
          fallback={
            <VoteIcon
              icon={BsHandThumbsUp}
              onClick={(e) => {
                e.preventDefault();
                void updateVote(props.queryId, 1);
              }}
            />
          }
          when={store[props.queryId] === 1}
        >
          <VoteIcon
            icon={BsHandThumbsUpFill}
            onClick={(e) => {
              e.preventDefault();
              void updateVote(props.queryId, 0);
            }}
          />
        </Show>

        <Show
          fallback={
            <VoteIcon
              icon={BsHandThumbsDown}
              onClick={(e) => {
                e.preventDefault();
                void updateVote(props.queryId, -1);
              }}
            />
          }
          when={store[props.queryId] === -1}
        >
          <VoteIcon
            icon={BsHandThumbsDownFill}
            onClick={(e) => {
              e.preventDefault();
              void updateVote(props.queryId, 0);
            }}
          />
        </Show>
      </div>
    </>
  );
};
