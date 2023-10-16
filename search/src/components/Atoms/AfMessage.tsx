/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { BiSolidUserRectangle } from "solid-icons/bi";
import { AiFillRobot } from "solid-icons/ai";
import { Accessor, Show } from "solid-js";

export interface AfMessageProps {
  role: "user" | "assistant" | "system";
  content: string;
  streamingCompletion: Accessor<boolean>;
}

export const AfMessage = (props: AfMessageProps) => {
  return (
    <>
      <Show when={props.role !== "system"}>
        <div
          classList={{
            "dark:text-white md:px-6 w-full px-4 py-4 flex items-start rounded-md":
              true,
            "bg-neutral-200 dark:bg-zinc-700": props.role === "assistant",
            "bg-neutral-50 dark:bg-zinc-800": props.role === "user",
          }}
        >
          <div class="w-full space-y-2 md:flex md:flex-row md:space-x-2 md:space-y-0 lg:space-x-4">
            {props.role === "user" ? (
              <BiSolidUserRectangle class="fill-current" />
            ) : (
              <AiFillRobot class="fill-current" />
            )}
            <div
              classList={{
                "w-full": true,
              }}
            >
              <div class="col-span-2 whitespace-pre-line text-neutral-800 dark:text-neutral-50">
                {props.content.trimStart()}
              </div>
              <Show when={!props.content}>
                <div class="col-span-2 w-full whitespace-pre-line">
                  <img
                    src="/cooking-crab.gif"
                    class="aspect-square w-[128px]"
                  />
                </div>
              </Show>
            </div>
          </div>
        </div>
      </Show>
    </>
  );
};
