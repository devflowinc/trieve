import { FaSolidTriangleExclamation } from "solid-icons/fa";
import { createEffect, createSignal, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { useTrieve } from "../hooks/useTrieve";
import {
  createUsageQuery,
  createSubscriptionQuery,
} from "../utils/fetchOrgUsage";

enum AlertState {
  Danger,
  Warning,
  Hidden,
}

const OrgUpdateAlert = () => {
  const [message, setMessage] = createSignal("");
  const [alertState, setAlertState] = createSignal(AlertState.Hidden);

  const userContext = useContext(UserContext);
  const trieve = useTrieve();

  const usageQuery = createUsageQuery(userContext, trieve);
  const subscriptionQuery = createSubscriptionQuery(userContext, trieve);

  const updateAlert = (
    currentCount: number,
    subscriptionLimit: number,
    variableName: string,
  ) => {
    const percentageUsed = ((currentCount / subscriptionLimit) * 100).toFixed(
      1,
    );

    if (currentCount >= subscriptionLimit) {
      setMessage(
        `Your organization has reached its total ${variableName} limit (${percentageUsed}% used).`,
      );
      setAlertState(AlertState.Danger);
    } else if (currentCount >= subscriptionLimit * 0.8) {
      setMessage(
        `Your organization is approaching its total ${variableName} limit (${percentageUsed}% used).`,
      );
      setAlertState(AlertState.Warning);
    } else {
      setMessage("");
      setAlertState(AlertState.Hidden);
    }
  };

  createEffect(() => {
    const orgUsage = usageQuery.data;
    const orgLimits = subscriptionQuery.data?.plan;

    if (!orgUsage || !orgLimits) return;

    const OrganizationUsageVariables = [
      {
        current: orgUsage.user_count,
        limit: orgLimits.user_count,
        name: "users",
      },
      {
        current: orgUsage.file_storage,
        limit: orgLimits.file_storage,
        name: "file storage",
      },
      {
        current: orgUsage.message_count,
        limit: orgLimits.message_count,
        name: "messages",
      },
      {
        current: orgUsage.chunk_count,
        limit: orgLimits.chunk_count,
        name: "chunks",
      },
    ];

    const approachingUsage = OrganizationUsageVariables.find(
      ({ current, limit }) => current >= limit || current >= limit * 0.8,
    );

    if (approachingUsage) {
      updateAlert(
        approachingUsage.current,
        approachingUsage.limit,
        approachingUsage.name,
      );
    } else {
      setMessage("");
      setAlertState(AlertState.Hidden);
    }
  });

  return (
    <div>
      {alertState() !== AlertState.Hidden && (
        <div
          class={`flex flex-row items-center justify-between rounded-lg border-2 bg-transparent bg-white p-4 ${
            alertState() ? "border-yellow-500" : "border-red-500"
          }`}
        >
          <div
            class={`flex flex-row items-center justify-center gap-3 ${
              alertState() ? "text-yellow-600" : "text-red-600"
            } `}
          >
            <FaSolidTriangleExclamation />
            <span class="text-sm font-semibold">
              {message()}
              {" To upgrade your subscription, click "}
              <a href="/org/billing" class="text-blue-500 underline">
                here
              </a>
              {' or visit the "Billing" tab.'}
            </span>
          </div>
        </div>
      )}
    </div>
  );
};

export default OrgUpdateAlert;
