import { createSignal } from "solid-js";
import { useParams, useSearchParams } from "solid-start";
import { A } from "solid-start";
import { detectReferralToken, isActixApiDefaultError } from "~/types/actix-api";

const SetPassword = () => {
  const [searchParams] = useSearchParams();
  detectReferralToken(searchParams.t);

  const apiHost: string = import.meta.env.VITE_API_HOST as unknown as string;

  const params = useParams();
  const [getErrorMessage, setErrorMessage] = createSignal("");
  const [getPassword, setPassword] = createSignal("");
  const [getPasswordConfirmation, setPasswordConfirmation] = createSignal("");

  return (
    <div class="flex h-screen w-screen items-center justify-center bg-neutral-50 px-10 dark:bg-neutral-800">
      <div class="flex w-full max-w-sm flex-col space-y-2 text-neutral-900 dark:text-neutral-50">
        <a href="/" class="flex flex-col items-center">
          <img src="/Logo.png" alt="Arguflow Logo" class="mx-auto my-2" />
        </a>
        <div class="text-center text-2xl font-bold">
          <span class="py-2">Finish Registration for Arguflow Chat</span>
        </div>
        <div class="text-center text-red-500">{getErrorMessage()}</div>
        <form class="flex flex-col space-y-4">
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
          <div class="flex flex-col space-y-2">
            <label for="password">Verify Password</label>
            <input
              type="password"
              name="password_confirmation"
              id="password_confirmation"
              class="rounded border border-neutral-300 p-2 text-neutral-900 dark:border-neutral-700"
              value={getPasswordConfirmation()}
              onInput={(e) => setPasswordConfirmation(e.currentTarget.value)}
            />
          </div>
          <div class="w-full">
            <button
              type="submit"
              class="w-full rounded bg-neutral-200 p-2  dark:bg-neutral-700"
              onClick={(e) => {
                e.preventDefault();
                void fetch(`${apiHost}/register/${params.id}`, {
                  method: "POST",
                  headers: {
                    "Content-Type": "application/json",
                  },
                  body: JSON.stringify({
                    password: getPassword(),
                    password_confirmation: getPasswordConfirmation(),
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
                  window.location.href = "/auth/login";
                });
              }}
            >
              Finish Registration
            </button>
          </div>
        </form>
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
    </div>
  );
};

export default SetPassword;
