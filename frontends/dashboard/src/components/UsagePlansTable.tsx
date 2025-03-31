import { For } from "solid-js";
import { createEffect, createSignal, onCleanup } from "solid-js";
import {
  OrganizationAndSubAndPlan,
  StripePlan,
  StripeSubscription,
} from "shared/types";
import { StripeUsageBasedPlan } from "trieve-ts-sdk";
import { ActiveTag } from "./PlansTable";

export interface PlansTableProps {
  currentOrgSubPlan: OrganizationAndSubAndPlan | null;
}

export const UsagePlansTable = (props: PlansTableProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const [availableUsagePlans, setAvailableUsagePlans] = createSignal<
    StripeUsageBasedPlan[]
  >([]);
  const [currentPlan, setCurrentPlan] = createSignal<
    StripePlan | StripeUsageBasedPlan | null
  >(null);
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

  const refetchOrgSubPlan = async () => {
    const selectedOrgId = props.currentOrgSubPlan?.organization.id ?? "";

    const resp = await fetch(`${apiHost}/organization/${selectedOrgId}`, {
      credentials: "include",
      headers: {
        "TR-Organization": selectedOrgId,
      },
    });

    if (resp.ok) {
      const data = (await resp.json()) as OrganizationAndSubAndPlan;
      setCurrentSubscription(data.subscription ?? null);
      setCurrentPlan(data.plan ?? null);
    }
  };

  const updatePlan = async (plan: StripeUsageBasedPlan) => {
    const resp = await fetch(
      `${apiHost}/stripe/subscription_plan/${
        props.currentOrgSubPlan?.subscription?.id ?? ""
      }/${plan.id}`,
      {
        credentials: "include",
        method: "PATCH",
        headers: {
          "TR-Organization": props.currentOrgSubPlan?.organization.id ?? "",
        },
      },
    );

    if (resp.ok) {
      setCurrentPlan(plan);
    }

    void refetchOrgSubPlan();
  };

  return (
    <For each={availableUsagePlans()}>
      {(plan: StripeUsageBasedPlan) => {
        const curPlan = currentPlan();
        const curSub = currentSubscription();
        let actionButton = <ActiveTag text="Current Tier" />;

        if (plan.id !== curPlan?.id || curSub?.current_period_end != null) {
          if (
            curPlan?.id !== "00000000-0000-0000-0000-000000000000" &&
            curSub?.current_period_end == null
          ) {
            actionButton = (
              <button
                onClick={() => void updatePlan(plan)}
                classList={{
                  "w-fit px-4 py-2 bg-magenta-500 text-white font-semibold rounded-lg shadow-sm shadow-magenta-100/40":
                    true,
                }}
              >
                Upgrade
              </button>
            );
          } else {
            actionButton = (
              <a
                href={`${apiHost}/stripe/payment_link/${plan.id}/${props.currentOrgSubPlan?.organization.id}?usage_based=true`}
                class="w-fit rounded-lg bg-magenta-500 px-4 py-2 font-semibold text-white shadow-sm shadow-magenta-100/40"
              >
                Subscribe
              </a>
            );
          }
        }

        if (!plan.visible && plan.id != curPlan?.id) {
          return null;
        }

        return (
          <tr>
            <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium sm:pl-6">
              {plan.name}
            </td>
            <td class="whitespace-nowrap px-3 py-4 text-sm text-neutral-800">
              **Usage based
            </td>
            <td class="whitespace-nowrap px-3 py-4 text-sm text-neutral-800">
              No Limit
            </td>
            <td class="whitespace-nowrap px-3 py-4 text-sm text-neutral-800">
              No Limit
            </td>
            <td class="whitespace-nowrap px-3 py-4 text-sm text-neutral-800">
              No Limit
            </td>
            <td class="whitespace-nowrap px-3 py-4 text-sm text-neutral-800">
              No Limit
            </td>
            <td class="whitespace-nowrap px-3 py-4 text-sm text-neutral-800">
              No Limit
            </td>
            <td class="whitespace-nowrap px-3 py-4">{actionButton}</td>
          </tr>
        );
      }}
    </For>
  );
};
