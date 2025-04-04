import { createEffect, createSignal, useContext } from "solid-js";
import { ProgressBar } from "./ProgressBar";
import {
  formatNumberWithCommas,
  formatStorageBytes,
  formatStorageKb,
} from "../utils/formatNumbers";
import { UserContext } from "../contexts/UserContext";
import { useTrieve } from "../hooks/useTrieve";
import { createSubscriptionQuery } from "../utils/fetchOrgUsage";
import { addMonths, startOfMonth } from "date-fns";
import { formatDateForApi } from "../analytics/utils/formatDate";
import {
  ExtendedOrganizationUsageCount,
  OrganizationWithSubAndPlan,
} from "trieve-ts-sdk";

export interface OrganizationUsageOverviewProps {
  currentOrgSubPlan: OrganizationWithSubAndPlan | null;
}

export const OrganizationUsageOverview = (
  props: OrganizationUsageOverviewProps,
) => {
  const userContext = useContext(UserContext);
  const trieve = useTrieve();

  const [startDate, setStartDate] = createSignal(startOfMonth(new Date()));
  const [organizationUsage, setOrganizationUsage] =
    createSignal<ExtendedOrganizationUsageCount | null>(null);

  createEffect(() => {
    void trieve
      .fetch("/api/organization/usage/{organization_id}", "post", {
        organizationId: userContext.selectedOrg().id,
        data: {
          date_range: {
            gte: formatDateForApi(startDate()),
            lte: formatDateForApi(addMonths(startDate(), 1)),
          },
          v1_usage: false,
        },
      })
      .then((organization_usage) => {
        setOrganizationUsage(organization_usage);
      });
  });

  const subscriptionQuery = createSubscriptionQuery(userContext, trieve);

  createEffect(() => {
    if (props.currentOrgSubPlan?.subscription?.type === "usage_based") {
      setStartDate(
        new Date(
          `${props.currentOrgSubPlan.subscription.last_cycle_timestamp}Z`,
        ),
      );
    }
  });

  return (
    <div class="mb-3 grid grid-cols-1 gap-5 lg:grid-cols-4">
      <dl class="col-span-4 grid grid-cols-1 divide-y divide-gray-200 overflow-hidden rounded-lg border bg-white shadow md:grid-cols-2 md:divide-x md:divide-y-0">
        <div class="col-span-2 px-4 pt-5 text-xl font-bold">
          {/* Display Currennt Billing Month (the current month)*/}
          <div>
            {startDate().toLocaleString("default", {
              month: "long",
              year: "numeric",
              day: "numeric",
            })}{" "}
            -{" "}
            {addMonths(startDate(), 1).toLocaleString("default", {
              month: "long",
              year: "numeric",
              day: "numeric",
            })}{" "}
            Usage
          </div>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total Users</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(organizationUsage()?.user_count ?? 0)}
                {props.currentOrgSubPlan?.subscription?.type !==
                  "usage_based" && (
                  <span class="ml-2 text-sm font-medium text-neutral-600">
                    of{" "}
                    {formatNumberWithCommas(
                      subscriptionQuery.data?.plan?.user_count || 0,
                    )}
                  </span>
                )}
              </div>
              <ProgressBar
                width={"200px"}
                max={subscriptionQuery.data?.plan?.user_count || 0}
                progress={organizationUsage()?.user_count || 0}
              />
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total File Storage</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatStorageKb(organizationUsage()?.file_storage ?? 0)}
                {props.currentOrgSubPlan?.subscription?.type !==
                  "usage_based" && (
                  <span class="ml-2 text-sm font-medium text-neutral-600">
                    of{" "}
                    {formatStorageKb(
                      subscriptionQuery.data?.plan?.file_storage || 0,
                    )}{" "}
                  </span>
                )}
              </div>
              <ProgressBar
                width={"200px"}
                max={subscriptionQuery.data?.plan?.file_storage || 0}
                progress={organizationUsage()?.file_storage || 0}
              />
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total Message Count</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(
                  organizationUsage()?.message_count ?? 0,
                )}
                {props.currentOrgSubPlan?.subscription?.type !==
                  "usage_based" && (
                  <span class="ml-2 text-sm font-medium text-neutral-600">
                    of{" "}
                    {formatNumberWithCommas(
                      subscriptionQuery.data?.plan?.message_count ?? 0,
                    )}
                  </span>
                )}
              </div>
              <ProgressBar
                width={"200px"}
                max={subscriptionQuery.data?.plan?.message_count || 0}
                progress={organizationUsage()?.message_count || 0}
              />
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total Chunk Count</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(organizationUsage()?.chunk_count ?? 0)}
                {props.currentOrgSubPlan?.subscription?.type !==
                  "usage_based" && (
                  <span class="ml-2 text-sm font-medium text-neutral-600">
                    of{" "}
                    {formatNumberWithCommas(
                      subscriptionQuery.data?.plan?.chunk_count ?? 0,
                    )}
                  </span>
                )}
              </div>
              <ProgressBar
                width={"200px"}
                max={subscriptionQuery.data?.plan?.chunk_count || 0}
                progress={organizationUsage()?.chunk_count || 0}
              />
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6 md:!border-r">
          <dt class="text-base font-normal">Total Dataset Count</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(
                  organizationUsage()?.dataset_count ?? 0,
                )}
                {props.currentOrgSubPlan?.subscription?.type !==
                  "usage_based" && (
                  <span class="ml-2 text-sm font-medium text-neutral-600">
                    of{" "}
                    {formatNumberWithCommas(
                      subscriptionQuery.data?.plan?.dataset_count ?? 0,
                    )}
                  </span>
                )}
              </div>
              <ProgressBar
                width={"200px"}
                max={subscriptionQuery.data?.plan?.dataset_count || 0}
                progress={organizationUsage()?.dataset_count || 0}
              />
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6 md:!border-r">
          <dt class="text-base font-normal">Ingested Tokens</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(
                  organizationUsage()?.tokens_ingested ?? 0,
                )}
              </div>
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6 md:!border-r">
          <dt class="text-base font-normal">Bytes Ingested</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatStorageBytes(organizationUsage()?.bytes_ingested ?? 0)}
              </div>
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6 md:!border-r">
          <dt class="text-base font-normal">Search Tokens</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(
                  organizationUsage()?.search_tokens ?? 0,
                )}
              </div>
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6 md:!border-r">
          <dt class="text-base font-normal">Total Message Tokens</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(
                  organizationUsage()?.message_tokens ?? 0,
                )}
              </div>
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6 md:!border-r">
          <dt class="text-base font-normal">OCR pages processed</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(
                  organizationUsage()?.ocr_pages_ingested ?? 0,
                )}
              </div>
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6 md:!border-r">
          <dt class="text-base font-normal">Pages Scraped</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(
                  organizationUsage()?.website_pages_scraped ?? 0,
                )}
              </div>
            </div>
          </dd>
        </div>
      </dl>
    </div>
  );
};
