import { Show, createSignal } from "solid-js";
import { A, useSearchParams } from "solid-start";
import {
  detectReferralToken,
  getReferralTokenArray,
  isActixApiDefaultError,
} from "~/types/actix-api";

const Register = () => {
  const [searchParams] = useSearchParams();
  detectReferralToken(searchParams.t);

  const apiHost: string = import.meta.env.VITE_API_HOST as unknown as string;

  const [getErrorMessage, setErrorMessage] = createSignal("");
  const [getEmail, setEmail] = createSignal("");
  const [getEmailSent, setEmailSent] = createSignal("");
  const [getIsLoading, setIsLoading] = createSignal(false);

  return (
    <div class="flex h-screen w-screen items-center justify-center bg-neutral-50 px-10 dark:bg-neutral-800">
      <div class="flex w-full max-w-sm flex-col space-y-2 text-neutral-900 dark:text-neutral-50">
        <A href="/" class="flex flex-col items-center">
          <img
            src="/Logo.png"
            alt="Arguflow Logo"
            class="mx-auto my-2"
            elementtiming={""}
            fetchpriority={"auto"}
          />
        </A>
        <Show when={!getEmailSent()}>
          <div class="text-center text-2xl font-bold">
            <span class="py-2">Register for Arguflow Chat</span>
          </div>
          <div class="text-center text-red-500">{getErrorMessage()}</div>
          <Show when={getErrorMessage().toLowerCase().includes("already")}>
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
                class="rounded border border-neutral-300 p-2 text-neutral-900 dark:border-neutral-700 "
                onInput={(e) => {
                  setEmail(e.currentTarget.value);
                }}
                value={getEmail()}
              />
            </div>
            <div class="w-full">
              <button
                type="submit"
                classList={{
                  "w-full rounded bg-neutral-200 p-2  dark:bg-neutral-700":
                    true,
                  "animate-pulse": getIsLoading(),
                }}
                onClick={(e) => {
                  e.preventDefault();
                  setIsLoading(true);
                  setErrorMessage("");
                  const email = getEmail();
                  void fetch(`${apiHost}/invitation`, {
                    method: "POST",
                    headers: {
                      "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                      email: email,
                      referral_tokens: getReferralTokenArray(),
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

                    void response.json().then((data) => {
                      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
                      setEmailSent(data.registration_url);
                    });
                  });
                }}
              >
                Register
              </button>
            </div>
          </form>
          <div class="flex w-full justify-center">
            <span class="">
              Already have an account? {` `}
              <A href="/auth/login" class="text-blue-500 hover:text-blue-600">
                Login
              </A>
            </span>
          </div>
        </Show>
        <Show when={getEmailSent()}>
          <div class="flex w-full max-w-sm flex-col space-y-2 p-2 text-neutral-900 dark:text-neutral-50">
            <div class="text-center text-2xl font-bold">
              <A
                href={`${getEmailSent()}`}
                class="py-2 text-blue-500 underline hover:text-blue-600"
              >
                Click here to set your password
              </A>
            </div>
            <div class="flex w-full justify-center">
              <span class="">
                Already have an account? {` `}
                <A
                  href="/auth/login"
                  class="text-blue-500 underline hover:text-blue-600"
                >
                  Login
                </A>
              </span>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
};

export default Register;
