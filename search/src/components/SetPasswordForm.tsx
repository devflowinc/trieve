import { createSignal } from "solid-js";
import { isActixApiDefaultError } from "../../utils/apiTypes";

const SetPasswordForm = (params: { id: string }) => {
  const api_host: string = import.meta.env.PUBLIC_API_HOST as unknown as string;

  const [getErrorMessage, setErrorMessage] = createSignal("");
  const [getPassword, setPassword] = createSignal("");
  const [getPasswordConfirmation, setPasswordConfirmation] = createSignal("");
  const [getIsLoading, setIsLoading] = createSignal(false);

  return (
    <>
      <div class="flex w-full max-w-sm flex-col space-y-2 p-2">
        <div class="text-center text-2xl font-bold">
          <span class="py-2">Finish Registration</span>
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
              classList={{
                "w-full rounded bg-neutral-200 p-2  dark:bg-neutral-700": true,
                "animate-pulse": getIsLoading(),
              }}
              onClick={(e) => {
                e.preventDefault();
                setIsLoading(true);
                void fetch(`${api_host}/register/${params.id}`, {
                  method: "POST",
                  headers: {
                    "Content-Type": "application/json",
                  },
                  body: JSON.stringify({
                    password: getPassword(),
                    password_confirmation: getPasswordConfirmation(),
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
            <a
              href="/auth/login"
              class="text-blue-500 underline hover:text-blue-600"
            >
              Login
            </a>
          </span>
        </div>
      </div>
    </>
  );
};

export default SetPasswordForm;
