import { Show, createSignal } from "solid-js";
import { isActixApiDefaultError } from "../../utils/apiTypes";

const LoginForm = () => {
  const [getEmail, setEmail] = createSignal("");
  const [getPassword, setPassword] = createSignal("");
  const [getErrorMessage, setErrorMessage] = createSignal("");
  const [getIsLoading, setIsLoading] = createSignal(false);

  const api_host: string = import.meta.env.PUBLIC_API_HOST as unknown as string;

  return (
    <>
      <div class="flex w-full max-w-sm flex-col space-y-2 p-2">
        <div class="text-center text-2xl font-bold">
          <span class="py-2">Login to Arguflow</span>
        </div>
        <div class="text-center text-red-500">{getErrorMessage()}</div>
        <Show when={getErrorMessage().toLowerCase().includes("incorrect")}>
          <div class="text-center text-sm ">
            Trouble signing in?{` `}
            <a class="text-blue-500 underline" href="/auth/password/reset">
              Reset your password
            </a>
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
              classList={{
                "w-full rounded bg-neutral-200 p-2  dark:bg-neutral-700": true,
                "animate-pulse": getIsLoading(),
              }}
              onClick={(e) => {
                e.preventDefault();
                setIsLoading(true);
                void fetch(`${api_host}/auth`, {
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
                  setIsLoading(false);
                  if (!response.ok) {
                    void response.json().then((data) => {
                      if (isActixApiDefaultError(data)) {
                        setErrorMessage(data.message);
                      }
                    });
                    return;
                  }
                  window.location.href = "/";
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
            <a
              href="/auth/register"
              class="text-blue-500 underline hover:text-blue-600"
            >
              Register
            </a>
          </span>
        </div>
        <div class="flex w-full justify-center">
          <span class="">
            <a
              href="/auth/password/reset"
              class="text-blue-500 underline hover:text-blue-600"
            >
              Reset password
            </a>
          </span>
        </div>
      </div>
    </>
  );
};

export default LoginForm;
