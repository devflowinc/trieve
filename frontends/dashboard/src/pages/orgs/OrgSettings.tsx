/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-argument */
import { Switch, createSignal, useContext, Match } from "solid-js";
import { UserContext } from "../../contexts/UserContext";
import { createToast } from "../../components/ShowToasts";
import { PartnerConfiguration } from "trieve-ts-sdk";

const OrgSettingsForm = () => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);

  const [organizationName, setOrganizationName] = createSignal<string>(
    userContext.selectedOrg().name,
  );
  const [partnerConfiguration, setPartnerConfiguration] =
    createSignal<PartnerConfiguration>(
      userContext.selectedOrg().partner_configuration as PartnerConfiguration,
    );

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        void fetch(`${apiHost}/organization`, {
          credentials: "include",
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
            "TR-Organization": userContext.selectedOrg().id,
          },
          body: JSON.stringify({
            name: organizationName(),
            partner_configuration: partnerConfiguration(),
          }),
        }).then(() => {
          createToast({
            title: "Success",
            message: "Organization updated successfully!",
            type: "success",
          });
          void userContext.login();
        });
      }}
    >
      <div class="border border-[#e5e7eb] shadow sm:overflow-hidden sm:rounded-md">
        <div class="bg-white px-4 py-6 sm:p-6">
          <div>
            <h2
              id="organization-details-name"
              class="text-lg font-medium leading-6"
            >
              General
            </h2>
            <p class="text-sm text-neutral-600">
              Update your organization's information.
            </p>
          </div>

          <div class="mt-4 grid grid-cols-4 gap-6">
            <div class="col-span-4 sm:col-span-2">
              <label
                for="organization-name"
                class="block text-sm font-medium leading-6"
              >
                Organization name
              </label>
              <input
                type="text"
                name="organization-name"
                id="organization-name"
                class="mt-0 block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-2 focus:ring-inset focus:ring-neutral-900 sm:text-sm sm:leading-6"
                value={organizationName()}
                onInput={(e) => setOrganizationName(e.currentTarget.value)}
              />
            </div>
          </div>
        </div>

        <div class="bg-white px-4 py-6 sm:p-6">
          <div>
            <h2
              id="organization-details-name"
              class="text-lg font-medium leading-6"
            >
              Partner Configuration
            </h2>
            <p class="text-sm text-neutral-600">
              Set your organization's partner settings here. Fields which are
              empty will not be shown.
            </p>
          </div>

          <div class="mt-4 grid grid-cols-4 gap-6">
            <div class="col-span-4 sm:col-span-2">
              <label
                for="organization-name"
                class="block text-sm font-medium leading-6"
              >
                Company name
              </label>
              <input
                type="text"
                name="company-name"
                id="company-name"
                class="mt-0 block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-2 focus:ring-inset focus:ring-neutral-900 sm:text-sm sm:leading-6"
                value={partnerConfiguration()?.COMPANY_NAME}
                onInput={(e) =>
                  setPartnerConfiguration({
                    ...partnerConfiguration(),
                    COMPANY_NAME: e.currentTarget.value,
                  })
                }
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="organization-name"
                class="block text-sm font-medium leading-6"
              >
                Company URL
              </label>
              <input
                type="text"
                name="company-name"
                id="company-name"
                class="mt-0 block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-2 focus:ring-inset focus:ring-neutral-900 sm:text-sm sm:leading-6"
                value={partnerConfiguration()?.COMPANY_URL}
                onInput={(e) =>
                  setPartnerConfiguration({
                    ...partnerConfiguration(),
                    COMPANY_URL: e.currentTarget.value,
                  })
                }
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="organization-name"
                class="block text-sm font-medium leading-6"
              >
                Favicon URL
              </label>
              <input
                type="text"
                name="favicon-url"
                id="favicon-url"
                class="mt-0 block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-2 focus:ring-inset focus:ring-neutral-900 sm:text-sm sm:leading-6"
                value={partnerConfiguration()?.FAVICON_URL}
                onInput={(e) =>
                  setPartnerConfiguration({
                    ...partnerConfiguration(),
                    FAVICON_URL: e.currentTarget.value,
                  })
                }
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="organization-name"
                class="block text-sm font-medium leading-6"
              >
                Demo Domain
              </label>
              <input
                type="text"
                name="demo-domain"
                id="demo-domain"
                class="mt-0 block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-2 focus:ring-inset focus:ring-neutral-900 sm:text-sm sm:leading-6"
                value={partnerConfiguration()?.DEMO_DOMAIN}
                onInput={(e) =>
                  setPartnerConfiguration({
                    ...partnerConfiguration(),
                    DEMO_DOMAIN: e.currentTarget.value,
                  })
                }
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="organization-name"
                class="block text-sm font-medium leading-6"
              >
                Calendar Link
              </label>
              <input
                type="text"
                name="calendar-link"
                id="calendar-link"
                class="mt-0 block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-2 focus:ring-inset focus:ring-neutral-900 sm:text-sm sm:leading-6"
                value={partnerConfiguration()?.CALENDAR_LINK}
                onInput={(e) =>
                  setPartnerConfiguration({
                    ...partnerConfiguration(),
                    CALENDAR_LINK: e.currentTarget.value,
                  })
                }
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="organization-name"
                class="block text-sm font-medium leading-6"
              >
                Slack Link
              </label>
              <input
                type="text"
                name="slack-link"
                id="slack-link"
                class="mt-0 block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-2 focus:ring-inset focus:ring-neutral-900 sm:text-sm sm:leading-6"
                value={partnerConfiguration()?.SLACK_LINK}
                onInput={(e) =>
                  setPartnerConfiguration({
                    ...partnerConfiguration(),
                    SLACK_LINK: e.currentTarget.value,
                  })
                }
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="organization-name"
                class="block text-sm font-medium leading-6"
              >
                LinkedIn Link
              </label>
              <input
                type="text"
                name="linkedin-link"
                id="linkedin-link"
                class="mt-0 block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-2 focus:ring-inset focus:ring-neutral-900 sm:text-sm sm:leading-6"
                value={partnerConfiguration()?.LINKEDIN_LINK}
                onInput={(e) =>
                  setPartnerConfiguration({
                    ...partnerConfiguration(),
                    LINKEDIN_LINK: e.currentTarget.value,
                  })
                }
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="organization-name"
                class="block text-sm font-medium leading-6"
              >
                Email
              </label>
              <input
                type="text"
                name="company-email"
                id="company-email"
                class="mt-0 block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-2 focus:ring-inset focus:ring-neutral-900 sm:text-sm sm:leading-6"
                value={partnerConfiguration()?.EMAIL}
                onInput={(e) =>
                  setPartnerConfiguration({
                    ...partnerConfiguration(),
                    EMAIL: e.currentTarget.value,
                  })
                }
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="organization-name"
                class="block text-sm font-medium leading-6"
              >
                Phone
              </label>
              <input
                type="text"
                name="company-phone"
                id="company-phone"
                class="mt-0 block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-2 focus:ring-inset focus:ring-neutral-900 sm:text-sm sm:leading-6"
                value={partnerConfiguration()?.PHONE}
                onInput={(e) =>
                  setPartnerConfiguration({
                    ...partnerConfiguration(),
                    PHONE: e.currentTarget.value,
                  })
                }
              />
            </div>
          </div>
        </div>

        <div class="border-t bg-neutral-50 px-4 py-3 text-right">
          <button
            type="submit"
            classList={{
              "inline-flex text-sm justify-center rounded-md bg-magenta-500 px-3 py-2 font-semibold text-white shadow-sm hover:bg-magenta-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-900":
                true,
            }}
          >
            Save
          </button>
        </div>
      </div>
    </form>
  );
};

const OrgDangerZoneForm = () => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);

  const [deleting, setDeleting] = createSignal<boolean>(false);

  const [confirmText, setConfirmText] = createSignal("");

  const deleteOrganization = () => {
    if (confirmText() !== userContext.selectedOrg().name) {
      createToast({
        title: "Error",
        message: "Organization name does not match!",
        type: "error",
      });
      return;
    }

    const confirmBox = confirm(
      "Deleting this organization will remove all chunks and all of your datasets. Are you sure you want to delete?",
    );
    if (!confirmBox) return;

    setDeleting(true);
    fetch(`${apiHost}/organization/${userContext.selectedOrg().id}`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
        "TR-Organization": userContext.selectedOrg().id,
      },
      credentials: "include",
    })
      .then((res) => {
        setDeleting(false);

        if (res.ok) {
          void userContext.login();

          createToast({
            title: "Success",
            message: "Organization deleted successfully!",
            type: "success",
          });

          return;
        }

        throw new Error("Error deleting organization!");
      })
      .catch(() => {
        setDeleting(false);

        createToast({
          title: "Error",
          message: "Error deleting organization!",
          type: "error",
        });
      });
  };

  return (
    <form class="rounded-md border border-red-600/20 shadow-sm shadow-red-500/30">
      <div class="shadow sm:overflow-hidden sm:rounded-md">
        <div class="space-y-3 bg-white px-3 py-6 sm:p-6">
          <div>
            <h2 id="user-details-name" class="text-lg font-medium leading-6">
              Delete Organization
            </h2>
            <p class="mt-0 text-sm text-red-700">
              Warning: This action is not reversible. Please be sure before
              deleting.
            </p>
            <div class="mt-3 grid grid-cols-4 gap-0">
              <div class="col-span-4 sm:col-span-2">
                <label
                  for="organization-name"
                  class="block text-sm font-medium leading-6 opacity-70"
                >
                  Enter the organization name
                  <span class="font-bold">
                    {` ${userContext.selectedOrg().name} `}
                  </span>
                  to confirm.
                </label>
                <input
                  type="text"
                  name="organization-name"
                  id="organization-name"
                  class="block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-inset focus:ring-neutral-900/20 sm:text-sm sm:leading-6"
                  value={confirmText()}
                  onInput={(e) => setConfirmText(e.currentTarget.value)}
                />
              </div>
            </div>
          </div>
        </div>
        <div class="border-t border-red-600/30 bg-red-50/40 px-3 py-3 text-right sm:px-3">
          <button
            onClick={() => {
              deleteOrganization();
            }}
            disabled={
              deleting() || confirmText() !== userContext.selectedOrg().name
            }
            classList={{
              "pointer:cursor text-sm w-fit disabled:opacity-50 font-bold rounded-md bg-red-600/80 border px-4 py-2 text-white hover:bg-red-500 focus:outline-magenta-500":
                true,
              "animate-pulse cursor-not-allowed": deleting(),
            }}
          >
            <Switch>
              <Match when={deleting()}>Deleting...</Match>
              <Match when={!deleting()}>Delete Organization</Match>
            </Switch>
          </button>
        </div>
      </div>
    </form>
  );
};

export const OrgSettings = () => {
  return (
    <div class="h-full pb-4">
      <div class="space-y-6 sm:px-6 lg:grid lg:grid-cols-2 lg:px-0">
        <section
          class="lg:col-span-2"
          aria-labelledby="organization-details-name"
        >
          <OrgSettingsForm />
        </section>
        <section class="lg:col-span-2" aria-labelledby="user-details-name">
          <OrgDangerZoneForm />
        </section>
      </div>
    </div>
  );
};
