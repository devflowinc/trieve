import {
  Menu,
  MenuItem,
  Popover,
  PopoverButton,
  PopoverPanel,
  Transition,
} from "solid-headless";
import { createEffect, createSignal } from "solid-js";
import { BsMoonStars, BsSun } from "solid-icons/bs";
import { CgScreen } from "solid-icons/cg";

export const setThemeMode = (mode: "light" | "dark") => {
  const oppositeMode = mode === "light" ? "dark" : "light";
  document.documentElement.classList.remove(oppositeMode);
  window.localStorage.setItem("theme", mode);
  document.documentElement.classList.add(mode);
};

export const getThemeMode = () => {
  const mode = window.localStorage.getItem("theme");
  return mode ?? "system";
};

export const clearThemeMode = () => {
  window.localStorage.removeItem("theme");
  if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
  }
};

const ThemeModeController = () => {
  const [currentTheme, setCurrentTheme] = createSignal(getThemeMode());

  createEffect(() => {
    const currentThemeMode = getThemeMode();

    if (
      currentThemeMode == "dark" ||
      (currentThemeMode === "system" &&
        window.matchMedia("(prefers-color-scheme: dark)").matches)
    ) {
      document.documentElement.classList.add("dark");
    }
  });

  return (
    <Transition
      show={true}
      enter="transition duration-2000"
      enterFrom="opacity-0"
      enterTo="opacity-100"
      leave="transition duration-2000"
      leaveFrom="opacity-100"
      leaveTo="opacity-0"
    >
      <div class="flex w-fit items-center justify-center">
        <Popover defaultOpen={false} class="relative flex items-center">
          {({ isOpen }) => (
            <>
              <PopoverButton
                aria-label="Toggle theme mode"
                classList={{
                  "text-neutral-500": currentTheme() === "system",
                  "text-indigo-500": currentTheme() !== "system",
                }}
              >
                <div class="hidden dark:block">
                  <BsMoonStars class=" h-6 w-6 fill-current" />
                </div>
                <div class="block dark:hidden">
                  <BsSun class=" h-6 w-6 fill-current" />
                </div>
              </PopoverButton>
              <Transition
                show={isOpen()}
                enter="transition duration-200"
                enterFrom="opacity-0"
                enterTo="opacity-100"
                leave="transition duration-150"
                leaveFrom="opacity-100"
                leaveTo="opacity-0"
              >
                <PopoverPanel
                  unmount={true}
                  class="absolute z-10 -translate-x-[60%] -translate-y-[115%] transform px-4 sm:px-0"
                >
                  <Menu class="flex flex-col space-y-1 overflow-hidden rounded-lg border border-slate-900 bg-neutral-100 p-1 shadow-lg drop-shadow-lg dark:bg-neutral-700 dark:text-white">
                    <MenuItem as="button" aria-label="Empty" />
                    <MenuItem
                      as="div"
                      class="flex space-x-2 rounded-md px-2 py-1 hover:cursor-pointer focus:bg-neutral-100 focus:outline-none dark:hover:bg-neutral-600 dark:hover:bg-none dark:focus:bg-neutral-600"
                      onClick={() => {
                        setThemeMode("light");
                        setCurrentTheme(getThemeMode());
                      }}
                    >
                      <div
                        classList={{
                          "text-neutral-500": currentTheme() === "system",
                          "text-violet-500": currentTheme() === "light",
                        }}
                      >
                        <BsSun class="h-6 w-6 fill-current" />
                      </div>
                      <div>
                        <div
                          classList={{
                            "text-md font-medium": true,
                            "text-black dark:text-white":
                              currentTheme() !== "light",
                            "text-violet-500": currentTheme() === "light",
                          }}
                        >
                          Light
                        </div>
                      </div>
                    </MenuItem>
                    <MenuItem
                      as="div"
                      class="flex space-x-2 rounded-md px-2 py-1 hover:cursor-pointer hover:bg-neutral-100 focus:bg-neutral-100 focus:outline-none dark:hover:bg-neutral-600 dark:focus:bg-neutral-600"
                      onClick={() => {
                        setThemeMode("dark");
                        setCurrentTheme(getThemeMode());
                      }}
                    >
                      <div
                        classList={{
                          "text-neutral-500": currentTheme() === "system",
                          "text-violet-500": currentTheme() === "dark",
                        }}
                      >
                        <BsMoonStars class="h-6 w-6 fill-current" />
                      </div>
                      <div>
                        <div
                          classList={{
                            "text-md font-medium": true,
                            "text-black dark:text-white":
                              currentTheme() !== "dark",
                            "text-violet-500": currentTheme() === "dark",
                          }}
                        >
                          Dark
                        </div>
                      </div>
                    </MenuItem>
                    <MenuItem
                      as="div"
                      class="flex space-x-2 rounded-md px-2 py-1 hover:cursor-pointer hover:bg-neutral-100 focus:bg-neutral-100 focus:outline-none dark:hover:bg-neutral-600 dark:focus:bg-neutral-600"
                      onClick={() => {
                        clearThemeMode();
                        setCurrentTheme(getThemeMode());
                      }}
                    >
                      <div
                        classList={{
                          "text-violet-500": currentTheme() === "system",
                        }}
                      >
                        <CgScreen class="h-6 w-6 fill-current" />
                      </div>
                      <div>
                        <div
                          classList={{
                            "text-md font-medium": true,
                            "text-black dark:text-white":
                              currentTheme() !== "system",
                            "text-violet-500": currentTheme() === "system",
                          }}
                        >
                          System
                        </div>
                      </div>
                    </MenuItem>
                  </Menu>
                </PopoverPanel>
              </Transition>
            </>
          )}
        </Popover>
      </div>
    </Transition>
  );
};

export default ThemeModeController;
