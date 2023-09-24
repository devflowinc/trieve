import { BiRegularMenuAltLeft, BiRegularPlus } from "solid-icons/bi";
import { Setter } from "solid-js";
import { Topic } from "~/types/topics";

export interface NavbarProps {
  setSideBarOpen: Setter<boolean>;
  selectedTopic: () => Topic | undefined;
  isCreatingTopic: () => boolean;
  setIsCreatingTopic: Setter<boolean>;
  isCreatingNormalTopic: () => boolean;
  setIsCreatingNormalTopic: Setter<boolean>;
}

export const Navbar = (props: NavbarProps) => {
  return (
    <div class="flex w-full items-center justify-between border-b border-neutral-200 px-5 py-2 font-semibold text-neutral-800 dark:border-neutral-800 dark:text-white md:text-xl">
      <div class="lg:hidden">
        <BiRegularMenuAltLeft
          onClick={() => props.setSideBarOpen((prev) => !prev)}
          class="fill-current text-4xl"
        />
      </div>
      <div class="flex w-full items-center justify-center px-2 text-center">
        <p>
          {props.selectedTopic()?.resolution ??
            (props.isCreatingNormalTopic()
              ? "New Normal Chat"
              : "New Retrieval Augmented Chat")}
        </p>
      </div>
      <div class="lg:hidden">
        <BiRegularPlus
          onClick={() => {
            props.setSideBarOpen(false);
            props.setIsCreatingTopic(true);
            props.setIsCreatingNormalTopic(false);
          }}
          class="fill-current text-4xl"
        />
      </div>
    </div>
  );
};
