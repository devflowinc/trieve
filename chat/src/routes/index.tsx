import {
  BiLogosGithub,
  BiLogosTwitch,
  BiLogosTwitter,
  BiLogosYoutube,
} from "solid-icons/bi";
import { createEffect, createSignal } from "solid-js";
import { A, useSearchParams } from "solid-start";
import ThemeModeController from "~/components/Navbar/ThemeModeController";
import { detectReferralToken } from "~/types/actix-api";

export default function Home() {
  const api_host: string = import.meta.env.VITE_API_HOST as unknown as string;

  const [searchParams] = useSearchParams();
  const [isLogin, setIsLogin] = createSignal<boolean>(false);

  detectReferralToken(searchParams.t);

  createEffect(() => {
    const abort_controller = new AbortController();

    void fetch(`${api_host}/auth`, {
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

    return () => {
      abort_controller.abort();
    };
  });

  return (
    <div class="bg-neutral-50 text-neutral-900 dark:bg-neutral-900 dark:text-neutral-50">
      <div class="bg-gradient-radial-t from-magenta-400 p-4">
        <div class="flex items-center justify-between rounded-lg bg-neutral-50 px-4 py-3 shadow-md dark:bg-neutral-800 lg:m-auto lg:max-w-5xl">
          <div class="flex items-center">
            <img
              class="w-10"
              src="/logo_transparent.svg"
              alt="Logo"
              elementtiming={""}
              fetchpriority={"high"}
            />
            <p class="text-lg">
              <span>Arguflow </span>
              <span class="text-magenta">Chat</span>
            </p>
          </div>
          <div class="flex items-center gap-4">
            <div class="hidden items-center gap-4 md:flex">
              <a class="hover:underline" href="https://arguflow.ai/">
                Home
              </a>
              <a class="hover:underline" href="https://search.arguflow.ai/">
                Evidence Search
              </a>
              <a class="hover:underline" href="https://blog.arguflow.ai/">
                Blog
              </a>
            </div>
            <ThemeModeController />
            <A
              class="rounded-lg bg-turquoise px-4 py-2 dark:text-neutral-900"
              href={isLogin() ? "/debate" : "/register"}
            >
              Start Debating
            </A>
          </div>
        </div>
        <div class="py-4" />
        <div class="flex flex-col items-center space-y-8">
          <p class="text-5xl md:text-6xl">
            <span>Arguflow </span>
            <span class="text-magenta">Chat</span>
          </p>
          <p class="text-center text-lg">
            Demo of Arguflow's LLM-chat infrastructure for a retrieval augmented
            AI debate opponent
          </p>
          <A
            class="rounded-lg bg-gradient-to-br from-cyan-900 to-turquoise px-4 py-2 text-white shadow-md"
            href={"https://arguflow.ai/meet"}
          >
            Get a Custom Solution
          </A>
        </div>
      </div>
      <div class="py-4" />
      <div class="flex w-full justify-center">
        <div class="w-fit px-4">
          <iframe
            class="h-[169px] w-[300px] md:h-[315px] md:w-[560px]"
            src="https://www.youtube.com/embed/RRJzvyKbM60"
            title="YouTube video player"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
            allowfullscreen
          />
        </div>
      </div>
      <footer class="mt-14 flex flex-col items-center bg-gradient-radial-b from-magenta pb-4 pt-20">
        <div class="flex items-center">
          <img
            class="w-14"
            src="/logo_transparent.svg"
            alt=""
            elementtiming={""}
            fetchpriority={"high"}
          />
          <p class="text-lg">
            <span>Arguflow </span>
            <span class="text-magenta">Chat</span>
          </p>
        </div>
        <div class="flex w-full flex-col  items-center gap-2">
          <a href="#pricing">Pricing</a>
          <a href="mailto:contact@arguflow.gg">Contact</a>
        </div>
        <div class="py-2" />
        <div class="flex gap-3">
          <a href="https://twitter.com/arguflowai" target="_blank">
            <BiLogosTwitter size={30} />
          </a>
          <a href="https://twitch.tv/arguflow" target="_blank">
            <BiLogosTwitch size={30} />
          </a>
          <a href="https://www.youtube.com/@arguflow">
            <BiLogosYoutube size={30} />
          </a>
          <a
            href="https://github.com/orgs/arguflow/repositories"
            target="_blank"
          >
            <BiLogosGithub size={30} />
          </a>
        </div>
      </footer>
    </div>
  );
}
