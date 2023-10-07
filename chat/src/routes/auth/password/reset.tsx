import { Show, createSignal } from "solid-js";
import { A, useSearchParams } from "solid-start";
import { detectReferralToken, isActixApiDefaultError } from "~/types/actix-api";

const ResetPassword = () => {
  const [searchParams] = useSearchParams();
  detectReferralToken(searchParams.t);

  const apiHost: string = import.meta.env.VITE_API_HOST as unknown as string;

  const [getErrorMessage, setErrorMessage] = createSignal("");
  const [getEmail, setEmail] = createSignal("");
  const [getEmailSent, setEmailSent] = createSignal(false);

  return (
    <div class="flex h-screen w-screen items-center justify-center bg-neutral-50 px-10 dark:bg-neutral-800">
      <div class="flex w-full max-w-sm flex-col space-y-2 text-neutral-900 dark:text-neutral-50">
        <a href="/" class="flex flex-col items-center">
          <img src="/Logo.png" alt="Arguflow Logo" class="mx-auto my-2" />
        </a>
        <Show when={!getEmailSent()}>
          <div class="text-center text-2xl font-bold">
            <span class="py-2">Reset Your Arguflow Chat Password</span>
          </div>
          <div class="text-center text-red-500">{getErrorMessage()}</div>
          <form class="flex flex-col space-y-4">
            <div class="flex flex-col space-y-2">
              <label for="email">Email</label>
              <input
                type="email"
                name="email"
                id="email"
                class="rounded border border-neutral-300 p-2 text-neutral-900 dark:border-neutral-700"
                onInput={(e) => {
                  setEmail(e.currentTarget.value);
                }}
                value={getEmail()}
              />
            </div>
            <div class="w-full">
              <button
                type="submit"
                class="w-full rounded bg-neutral-200 p-2  dark:bg-neutral-700"
                onClick={(e) => {
                  e.preventDefault();
                  setErrorMessage("");
                  const email = getEmail();
                  if (!email) {
                    setErrorMessage("Email is required");
                    return;
                  }
                  void fetch(`${apiHost}/password/${email}`, {
                    method: "GET",
                    headers: {
                      "Content-Type": "application/json",
                    },
                  }).then((response) => {
                    if (!response.ok) {
                      void response.json().then((data) => {
                        if (isActixApiDefaultError(data)) {
                          setErrorMessage(data.message);
                        }
                      });
                      return;
                    }
                    setEmailSent(true);
                  });
                }}
              >
                Send Email to Reset Password
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
        </Show>
        <Show when={getEmailSent()}>
          <div class="flex w-full max-w-sm flex-col space-y-2 text-neutral-900 dark:text-neutral-50">
            <div class="text-center text-2xl font-bold">
              <span class="py-2">
                Check your email to finish resetting your password
              </span>
            </div>
            <div class="text-center">
              Your password reset link will expire in 5 minutes
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
};

export default ResetPassword;
