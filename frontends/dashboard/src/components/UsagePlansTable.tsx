import { For, Show } from "solid-js";
import { createEffect, createSignal, onCleanup } from "solid-js";
import {
  OrganizationAndSubAndPlan,
  StripePlan,
  StripeSubscription,
} from "shared/types";
import { AiOutlineWarning } from "solid-icons/ai";
import { StripeUsageBasedPlan } from "trieve-ts-sdk";
import { ActiveTag } from "./PlansTable";

interface CreateSetupCheckoutSessionResPayload {
  url: string;
}

export interface PlansTableProps {
  currentOrgSubPlan: OrganizationAndSubAndPlan | null;
}

export const UsagePlansTable = (props: PlansTableProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const [availableUsagePlans, setAvailableUsagePlans] = createSignal<
    StripeUsageBasedPlan[]
  >([]);
  const [currentPlan, setCurrentPlan] = createSignal<StripePlan | StripeUsageBasedPlan | null>(null);
  const [currentSubscription, setCurrentSubscription] =
    createSignal<StripeSubscription | null>(null);

  createEffect(() => {
    setCurrentPlan(props.currentOrgSubPlan?.plan ?? null);
    setCurrentSubscription(props.currentOrgSubPlan?.subscription ?? null);
  });

  createEffect(() => {
    if (!props.currentOrgSubPlan?.organization) {
      return;
    }
    const availablePlansAbortController = new AbortController();
    void fetch(`${apiHost}/stripe/usage_plans`, {
      credentials: "include",
      headers: {
        "TR-Organization": props.currentOrgSubPlan?.organization.id ?? "",
      },
      signal: availablePlansAbortController.signal,
    })
      .then((res) => res.json())
      .then((data) => {
        setAvailableUsagePlans(data as StripeUsageBasedPlan[]);
      });

    onCleanup(() => {
      availablePlansAbortController.abort();
    });
  });

  const createStripeSetupCheckoutSession = () => {
    if (!props.currentOrgSubPlan?.subscription) {
      return;
    }
    const selectedOrgId = props.currentOrgSubPlan?.organization.id ?? "";
    if (selectedOrgId == "") {
      return;
    }

    const checkoutSessionAbortController = new AbortController();
    void fetch(
      `${apiHost}/stripe/checkout/setup/${selectedOrgId}?usage_based=true`,
      {
        method: "POST",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
          "TR-Organization": selectedOrgId,
        },
        signal: checkoutSessionAbortController.signal,
      },
    ).then((res) =>
      res.json().then((res) => {
        const data = res as CreateSetupCheckoutSessionResPayload;
        window.location.href = data.url;
      }),
    );
  };

  return (
    <div class="my-8 flex flex-col gap-8">
      <div class="space-y-2">
        <div class="flex w-full flex-row items-start justify-between">
          <div>
            <h3 class="text-lg font-semibold text-neutral-800">
              Current plan for {props.currentOrgSubPlan?.organization.name}{" "}
              Organization
            </h3>
            <Show when={currentSubscription()?.current_period_end}>
              {(end_date_str) => (
                <div class="flex space-x-2 bg-yellow-50 px-4 py-2">
                  <AiOutlineWarning class="block fill-current pt-1 text-yellow-500" />
                  <div>
                    <p class="font-semibold text-yellow-800">Notice</p>
                    <p class="text-sm">
                      This organization will lose the{" "}
                      <span class="font-semibold">{currentPlan()?.name}</span>{" "}
                      plan benefits on{" "}
                      {new Date(end_date_str()).toLocaleDateString()} and be
                      downgraded to the Free plan.
                    </p>
                  </div>
                </div>
              )}
            </Show>
          </div>
          <Show when={props.currentOrgSubPlan?.subscription}>
            <button
              onClick={() => {
                createStripeSetupCheckoutSession();
              }}
              class="w-fit rounded-lg bg-magenta-500 px-4 py-2 font-semibold text-white shadow-sm shadow-magenta-100/40"
            >
              Update payment method
            </button>
          </Show>
        </div>
        <div class="overflow-hidden rounded shadow ring-1 ring-black ring-opacity-5">
          <table class="min-w-full divide-y divide-neutral-300">
            <thead class="bg-neutral-100">
              <tr>
                <th
                  scope="col"
                  class="py-3.5 pl-4 text-left text-sm font-semibold sm:pl-6"
                >
                  Name
                </th>
                <th scope="col" class="px-3 py-3.5" />
              </tr>
            </thead>
            <tbody class="divide-y divide-neutral-200 bg-white">
              <For each={availableUsagePlans()}>
                {(plan: StripeUsageBasedPlan) => {
                  const curPlan = currentPlan();
                  let actionButton = <ActiveTag text="Current Tier" />;

                  if (plan.id !== curPlan?.id) {
                    actionButton = (
                      <a
                        href={`${apiHost}/stripe/payment_link/${plan.id}/${props.currentOrgSubPlan?.organization.id}?usage_based=true`}
                        class="w-fit rounded-lg bg-magenta-500 px-4 py-2 font-semibold text-white shadow-sm shadow-magenta-100/40"
                      >
                        Subscribe
                      </a>
                    );
                  }

                  if (!plan.visible && plan.id != curPlan?.id) {
                    return null;
                  }

                  return (
                    <tr>
                      <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium sm:pl-6">
                        {plan.name}
                      </td>
                      <td class="whitespace-nowrap px-3 py-4">
                        {actionButton}
                      </td>
                    </tr>
                  );
                }}
              </For>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
