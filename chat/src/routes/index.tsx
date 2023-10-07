/* eslint-disable prettier/prettier */
import { AiFillGithub } from "solid-icons/ai";
import {
  BiLogosGithub,
  BiLogosTwitch,
  BiLogosTwitter,
  BiLogosYoutube,
} from "solid-icons/bi";
import { TbMinusVertical } from "solid-icons/tb";
import { Show, createEffect, createSignal, onCleanup } from "solid-js";
import { A, useSearchParams } from "solid-start";
import ThemeModeController from "~/components/Navbar/ThemeModeController";
import { detectReferralToken } from "~/types/actix-api";

export default function Home() {
  const apiHost: string = import.meta.env.VITE_API_HOST as unknown as string;
  const searchURL = import.meta.env.VITE_SEARCH_URL as string;
  const dataset = import.meta.env.VITE_DATASET as unknown as string;
  const youtubeEmbedURL = import.meta.env.VITE_YOUTUBE_EMBED_URL as string;
  const showGithubStars = import.meta.env
    .VITE_SHOW_GITHUB_STARS as unknown as string;

  const [searchParams] = useSearchParams();
  const [isLogin, setIsLogin] = createSignal<boolean>(false);
  const [starCount, setStarCount] = createSignal(0);

  detectReferralToken(searchParams.t);

  createEffect(() => {
    const abort_controller = new AbortController();

    void fetch(`${apiHost}/auth`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      signal: abort_controller.signal,
    }).then((response) => {
      if (!response.ok) {
        setIsLogin(false);
        return;
      }
      setIsLogin(true);
    });

    onCleanup(() => {
      abort_controller.abort();
    });
  });

  createEffect(() => {
    try {
      void fetch(`https://api.github.com/repos/arguflow/arguflow`, {
        headers: {
          Accept: "application/vnd.github+json",
        },
      }).then((response) => {
        if (!response.ok) {
          return;
        }
        void response.json().then((data) => {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          setStarCount(data.stargazers_count);
        });
      });
    } catch (e) {
      console.error(e);
    }
  });

  return (
    <div class="flex min-h-screen flex-col bg-neutral-50 text-neutral-900 dark:bg-neutral-900 dark:text-neutral-50">
      <div class="bg-gradient-radial-t from-magenta-400 p-4">
        <div class="flex items-center justify-end rounded-lg bg-neutral-50 px-4 py-3 shadow-md dark:bg-neutral-800 sm:justify-between lg:m-auto lg:max-w-5xl">
          <div class="hidden items-center sm:flex">
            <img
              class="w-10"
              src="/logo_transparent.svg"
              alt="Logo"
              elementtiming={""}
              fetchpriority={"high"}
            />
            <div>
              <div class="mb-[-4px] w-full text-end align-bottom text-xs leading-3 text-turquoise">
                {dataset}
              </div>
              <div class="align-top text-lg">
                <span>Arguflow</span>
                <span class="text-magenta">Chat</span>\
              </div>
            </div>
          </div>
          <div class="flex items-center gap-4">
            <div class="hidden items-center gap-4 md:flex">
              <a class="hover:underline" href={searchURL}>
                Search
              </a>
              <a class="hover:underline" href="https://blog.arguflow.ai/">
                Blog
              </a>
            </div>
            <Show when={showGithubStars !== "off"}>
              <a href="https://github.com/arguflow/arguflow">
                <div class="flex items-center justify-center rounded border border-black px-2 py-1 hover:border-gray-300 hover:bg-gray-300 dark:border-white dark:hover:border-neutral-700 dark:hover:bg-neutral-700">
                  <AiFillGithub class="mr-2 h-[26px] w-[26px] fill-current" />
                  <p class="text-sm">STAR US</p>
                  <TbMinusVertical size={25} />
                  <p>{starCount()}</p>
                </div>
              </a>
            </Show>
            <ThemeModeController />
            <A
              class="rounded-lg bg-turquoise px-4 py-2 font-semibold dark:text-neutral-900"
              href={isLogin() ? "/chat" : "/register"}
            >
              Chat Now
            </A>
          </div>
        </div>
        <div class="py-4" />
        <div class="flex flex-col items-center space-y-8">
          <div>
            <div class="mb-[-4px] w-full text-end align-bottom text-lg leading-3 text-turquoise">
              {dataset}
            </div>
            <div class="text-5xl md:text-6xl">
              <span>Arguflow</span>
              <span class="text-magenta">Chat</span>
            </div>
          </div>
          <p class="text-center text-lg">
            Retrieval augmented LLM chatbot that lets you chat with{" "}
            <span class="text-turquoise"> {dataset}</span>
          </p>
          <A
            class="rounded-lg bg-turquoise px-4 py-2 font-semibold text-black shadow-md"
            href={"https://arguflow.ai/meet"}
          >
            Get a Custom Solution
          </A>
        </div>
      </div>
      <Show when={youtubeEmbedURL}>
        <div class="py-4" />
        <div class="flex w-full justify-center">
          <div class="w-fit px-4">
            <iframe
              class="h-[169px] w-[300px] md:h-[315px] md:w-[560px]"
              src={youtubeEmbedURL}
              title="YouTube video player"
              allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
              allowfullscreen
            />
          </div>
        </div>
      </Show>
      <div class="flex-1" />
      <footer class="mt-14 flex flex-col items-center bg-gradient-radial-b from-magenta pb-4 pt-20">
        <div class="flex items-center">
          <img
            class="w-14"
            src="/logo_transparent.svg"
            alt=""
            elementtiming={""}
            fetchpriority={"high"}
          />
          <div>
            <div class="mb-[-4px] w-full text-end align-bottom text-xs leading-3 text-turquoise">
              {dataset}
            </div>
            <div class="align-top text-lg">
              <span>Arguflow</span>
              <span class="text-magenta">Chat</span>\
            </div>
          </div>
        </div>
        <div class="flex w-full flex-col  items-center gap-2">
          <a href="mailto:contact@arguflow.gg">contact@arguflow.gg</a>
        </div>
        <div class="py-2" />
        <div class="flex gap-3">
          <a href="https://twitter.com/arguflowai" target="_blank">
            <BiLogosTwitter size={30} class="fill-current" />
          </a>
          <a href="https://twitch.tv/arguflow" target="_blank">
            <BiLogosTwitch size={30} class="fill-current" />
          </a>
          <a href="https://www.youtube.com/@arguflow">
            <BiLogosYoutube size={30} class="fill-current" />
          </a>
          <a
            href="https://github.com/orgs/arguflow/repositories"
            target="_blank"
          >
            <BiLogosGithub size={30} class="fill-current" />
          </a>
        </div>
      </footer>
    </div>
  );
}
