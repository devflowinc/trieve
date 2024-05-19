import { BiLogosDiscord, BiLogosGithub, BiLogosTwitter } from "solid-icons/bi";
import { SiMatrix } from "solid-icons/si";
import ThemeModeController from "./ThemeModeController";

export const Footer = () => {
  return (
    <div class="mt-12 flex w-full flex-col items-center space-y-2 py-4">
      <div class="flex w-full justify-center space-x-3">
        <a
          href="https://matrix.to/#/#trieve-general:matrix.zerodao.gg"
          target="_blank"
          class="hover:text-turquoise-500 dark:hover:text-acid-500"
        >
          <SiMatrix size={30} class="fill-current" />
        </a>
        <a
          href="https://discord.gg/CuJVfgZf54"
          target="_blank"
          class="hover:text-turquoise-500 dark:hover:text-acid-500"
        >
          <BiLogosDiscord size={30} class="fill-current" />
        </a>
        <a
          href="https://twitter.com/trieveai"
          target="_blank"
          class="hover:text-turquoise-500 dark:hover:text-acid-500"
        >
          <BiLogosTwitter size={30} class="fill-current" />
        </a>
        <a
          href="https://github.com/devflowinc/trieve"
          target="_blank"
          class="hover:text-turquoise-500 dark:hover:text-acid-500"
        >
          <BiLogosGithub size={30} class="fill-current" />
        </a>
      </div>
      <div class="flex w-full justify-center space-x-4">
        <div>humans@trieve.ai</div>
        <div>
          <ThemeModeController />
        </div>
      </div>
    </div>
  );
};
