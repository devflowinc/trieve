import { Transition } from "solid-headless";
import { Show, createEffect, createSignal } from "solid-js";
import {
  UserDTO,
  isActixApiDefaultError,
  isUserDTO,
} from "../../utils/apiTypes";

const SearchForm = () => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;

  const [currentUser, setCurrentUser] = createSignal<UserDTO | null>(null);
  const [username, setUsername] = createSignal("");
  const [website, setWebsite] = createSignal("");
  const [hideEmail, setHideEmail] = createSignal(false);
  const [errorText, setErrorText] = createSignal("");
  const [errorFields, setErrorFields] = createSignal<string[]>([]);
  const [isSubmitting, setIsSubmitting] = createSignal(false);

  const updateUser = (e: Event) => {
    e.preventDefault();
    setIsSubmitting(true);
    void fetch(`${apiHost}/user`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      body: JSON.stringify({
        username: username(),
        website: website(),
        visible_email: !hideEmail(),
      }),
    }).then((response) => {
      if (response.ok) {
        setErrorText("");
        setErrorFields([]);
        setIsSubmitting(false);
        return;
      }

      void response.json().then((data) => {
        if (!isActixApiDefaultError(data)) {
          setErrorText("An unknown error occurred.");
          setIsSubmitting(false);
          return;
        }

        setErrorText(data.message);
        const newErrorFields: string[] = [];
        data.message.toLowerCase().includes("username") &&
          newErrorFields.push("username");
        data.message.toLowerCase().includes("website") &&
          newErrorFields.push("website");
        data.message.toLowerCase().includes("email") &&
          newErrorFields.push("hideEmail");
        setErrorFields(newErrorFields);
        setIsSubmitting(false);
      });
    });
  };

  createEffect(() => {
    void fetch(`${apiHost}/auth`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          if (isUserDTO(data)) {
            setCurrentUser(data);
            setUsername(data.username ?? "");
            setWebsite(data.website ?? "");
            setHideEmail(!data.visible_email);
            return;
          }
          window.location.href = "/auth/register";
        });
        return;
      }
      window.location.href = "/auth/register";
    });
  });

  return (
    <>
      <Transition
        show={!currentUser()}
        enter="transition duration-400"
        enterFrom="opacity-0"
        enterTo="opacity-100"
        leave="transition duration-150"
        leaveFrom="opacity-100"
        leaveTo="opacity-0"
      >
        <div class="mx-auto mt-16 h-32 w-32 animate-spin rounded-full border-b-2 border-t-2 border-neutral-900 dark:border-white" />
      </Transition>
      <Transition
        show={!!currentUser()}
        enter="transition duration-600"
        enterFrom="opacity-0"
        enterTo="opacity-100"
        leave="transition duration-200"
        leaveFrom="opacity-100"
        leaveTo="opacity-0"
      >
        <form
          class="mb-8 h-full w-full text-neutral-800 dark:text-white"
          onSubmit={(e) => {
            e.preventDefault();
            updateUser(e);
          }}
        >
          <div class="text-center text-red-500">{errorText()}</div>
          <div class="mt-8 grid w-full grid-cols-[1fr,2fr] justify-items-end gap-x-8 gap-y-6">
            <div>Email</div>
            <div class="flex w-full justify-start">{currentUser()?.email}</div>

            <div>Username</div>
            <input
              type="text"
              value={username()}
              onInput={(e) => setUsername(e.target.value)}
              maxlength={20}
              classList={{
                "w-full bg-neutral-100 rounded-md px-4 py-1 dark:bg-neutral-700":
                  true,
                "border border-red-500": errorFields().includes("username"),
              }}
            />

            <div>Website</div>
            <input
              type="url"
              value={website()}
              onInput={(e) => setWebsite(e.target.value)}
              classList={{
                "w-full bg-neutral-100 rounded-md px-4 py-1 dark:bg-neutral-700":
                  true,
                "border border-red-500": errorFields().includes("website"),
              }}
            />

            <div>Hide Email</div>
            <div class="flex w-full justify-start">
              <input
                type="checkbox"
                checked={hideEmail()}
                onInput={(e) => setHideEmail(e.target.checked)}
                classList={{
                  "h-6 w-6": true,
                  "ring ring-red-500 ring-1":
                    errorFields().includes("hideEmail"),
                }}
              />
            </div>
          </div>

          <div class="mt-6 flex w-full justify-center space-x-2 border-t border-neutral-300 pt-6 dark:border-neutral-700">
            <button
              class="w-fit rounded bg-neutral-100 p-2 hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800"
              type="submit"
              disabled={isSubmitting()}
            >
              <Show when={!isSubmitting()}>Save Changes</Show>
              <Show when={isSubmitting()}>
                <div class="animate-pulse">Submitting...</div>
              </Show>
            </button>
          </div>
        </form>
      </Transition>
    </>
  );
};

export default SearchForm;
