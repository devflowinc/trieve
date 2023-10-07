import { Show, createSignal } from "solid-js";
import { A, useSearchParams } from "solid-start";
import { detectReferralToken, isActixApiDefaultError } from "~/types/actix-api";

const login = () => {
  const [searchParams] = useSearchParams();
  detectReferralToken(searchParams.t);

  const [getEmail, setEmail] = createSignal("");
  const [getPassword, setPassword] = createSignal("");
  const [getErrorMessage, setErrorMessage] = createSignal("");

  const apiHost: string = import.meta.env.VITE_API_HOST as unknown as string;

  return (
    <div class="flex h-screen w-screen items-center justify-center bg-neutral-50 px-10 text-neutral-900 dark:bg-neutral-800 dark:text-neutral-50">
      <div class="flex w-full max-w-sm flex-col space-y-2 ">
        <a href="/" class="flex flex-col items-center">
          <img src="/Logo.png" alt="Arguflow Logo" class="mx-auto my-2" />
        </a>
        <div class="text-center text-2xl font-bold">
          <span class="py-2">Login to Arguflow Chat</span>
        </div>
        <div class="text-center text-red-500">{getErrorMessage()}</div>
        <Show when={getErrorMessage().toLowerCase().includes("incorrect")}>
          <div class="text-center text-sm ">
            Trouble signing in?{` `}
            <A class="text-blue-500 underline" href="/auth/password/reset">
              Reset your password
            </A>
          </div>
        </Show>
        <form class="flex flex-col space-y-4">
          <div class="flex flex-col space-y-2">
            <label for="email">Email</label>
            <input
              type="email"
              name="email"
              id="email"
              class="rounded border border-neutral-300 p-2 text-neutral-900 dark:border-neutral-700"
              value={getEmail()}
              onInput={(e) => setEmail(e.currentTarget.value)}
            />
          </div>
          <div class="flex flex-col space-y-2">
            <label for="password">Password</label>
            <input
              type="password"
              name="password"
              id="password"
              class="rounded border border-neutral-300 p-2 text-neutral-900 dark:border-neutral-700"
              value={getPassword()}
              onInput={(e) => setPassword(e.currentTarget.value)}
            />
          </div>
          <div class="w-full">
            <button
              type="submit"
              class="w-full rounded bg-neutral-200 p-2  dark:bg-neutral-700"
              onClick={(e) => {
                e.preventDefault();
                void fetch(`${apiHost}/auth`, {
                  method: "POST",
                  headers: {
                    "Content-Type": "application/json",
                  },
                  credentials: "include",
                  body: JSON.stringify({
                    email: getEmail(),
                    password: getPassword(),
                  }),
                }).then((response) => {
                  if (!response.ok) {
                    void response.json().then((data) => {
                      if (isActixApiDefaultError(data)) {
                        setErrorMessage(data.message);
                      }
                    });
                    return;
                  }
                  window.location.href = "/chat";
                });
              }}
            >
              Login
            </button>
          </div>
        </form>
        <div class="flex w-full justify-center">
          <span class="">
            Don't have an account? {` `}
            <A
              href="/auth/register"
              class="text-blue-500 underline hover:text-blue-600"
            >
              Register
            </A>
          </span>
        </div>
      </div>
    </div>
  );
};

export default login;
