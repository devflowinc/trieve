import { Show, createEffect, createSignal } from "solid-js";
import { APIRequest, isActixApiDefaultError } from "../../utils/apiTypes";
import { currentUser } from "../stores/userStore";
import { useStore } from "@nanostores/solid";

const SearchForm = () => {
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const $currentUser = useStore(currentUser);
  const [username, setUsername] = createSignal("");
  const [website, setWebsite] = createSignal("");
  const [hideEmail, setHideEmail] = createSignal(false);
  const [errorText, setErrorText] = createSignal("");
  const [errorFields, setErrorFields] = createSignal<string[]>([]);
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [apiKey, setApiKey] = createSignal("");
  const [isGenerating, setIsGenerating] = createSignal(false);

  const generateAPIKey = () => (e: Event) => {
    e.preventDefault();
    setIsGenerating(true);
    void fetch(`${apiHost}/user/set_api_key`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      //TODO: Add modal to specify name
      body: JSON.stringify({
        "name": "search"
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          setApiKey((data as APIRequest).api_key);
          setIsGenerating(false);
        });
        return;
      }

      void response.json().then((data) => {
        if (!isActixApiDefaultError(data)) {
          setErrorText("An unknown error occurred.");
          return;
        }
        setIsGenerating(false);
        setErrorText(data.message);
      });
    });
  };

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
    currentUser.subscribe((user) => {
      if (user) {
        setUsername(user.username ?? "");
        setWebsite(user.website ?? "");
        setHideEmail(!user.visible_email);
        return;
      }
    });
  });

  return (
    <>
      <Show when={!$currentUser()}>
        <div class="mx-auto mt-16 h-32 w-32 animate-spin rounded-full border-b-2 border-t-2 border-neutral-900 dark:border-white" />
      </Show>
      <Show when={!!$currentUser()}>
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
            <div class="flex w-full justify-start">{$currentUser()?.email}</div>

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
                  "ring-red-500 ring-1": errorFields().includes("hideEmail"),
                }}
              />
            </div>
            <div>API Key</div>
            <div class="flex w-full justify-start">
              <Show when={!apiKey()}>
                <button
                  classList={{
                    "rounded bg-neutral-100 p-2 hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800":
                      true,
                    "animate-pulse": isGenerating(),
                  }}
                  onClick={generateAPIKey()}
                >
                  Generate
                </button>
              </Show>
              <Show when={apiKey()}>
                <div>{apiKey()}</div>
              </Show>
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
      </Show>
    </>
  );
};

export default SearchForm;
