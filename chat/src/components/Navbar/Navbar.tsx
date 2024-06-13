import { BiRegularMenuAltLeft, BiRegularPlus } from "solid-icons/bi";
import { Setter, Switch, Match } from "solid-js";
import { Topic } from "../../utils/apiTypes";

export interface NavbarProps {
  setSideBarOpen: Setter<boolean>;
  selectedTopic: () => Topic | undefined;
  isCreatingTopic: () => boolean;
  setIsCreatingTopic: Setter<boolean>;
  loadingNewTopic: boolean;
  setSelectedNewTopic: Setter<boolean>;
}

export const Navbar = (props: NavbarProps) => {
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
          <div class="flex w-full items-center justify-center px-2 text-center text-sm">
            <p>Loading...</p>
          </div>
        </Match>
        <Match when={!props.loadingNewTopic}>
          <div class="flex w-full items-center justify-center px-2 text-center text-sm">
            <p>{props.selectedTopic()?.name ?? "New RAG Chat"}</p>
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
