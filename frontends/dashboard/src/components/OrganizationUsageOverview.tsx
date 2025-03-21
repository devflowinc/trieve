import { useContext } from "solid-js";
import { ProgressBar } from "./ProgressBar";
import {
  formatNumberWithCommas,
  // formatStorageBytes,
  formatStorageMb,
} from "../utils/formatNumbers";
import { UserContext } from "../contexts/UserContext";
import { useTrieve } from "../hooks/useTrieve";
import {
  createUsageQuery,
  createSubscriptionQuery,
} from "../utils/fetchOrgUsage";
import { addMonths, startOfMonth } from "date-fns";
import { formatDateForApi } from "../analytics/utils/formatDate";

export const OrganizationUsageOverview = () => {
  const userContext = useContext(UserContext);
  const trieve = useTrieve();

  const usageQuery = createUsageQuery(userContext, trieve, {
    startDate: formatDateForApi(startOfMonth(new Date())),
    endDate: formatDateForApi(addMonths(startOfMonth(new Date()), 1)),
  });

  const subscriptionQuery = createSubscriptionQuery(userContext, trieve);

  return (
    <div class="mb-3 grid grid-cols-1 gap-5 lg:grid-cols-4">
      <dl class="col-span-4 grid grid-cols-1 divide-y divide-gray-200 overflow-hidden rounded-lg border bg-white shadow md:grid-cols-2 md:divide-x md:divide-y-0">
        <div class="col-span-2 px-4 pt-5 text-xl font-bold">
          {/* Display Currennt Billing Month (the current month)*/}
          <div>
            {new Date().toLocaleString("default", {
              month: "long",
              year: "numeric",
            })}{" "}
            Usage
          </div>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total Users</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(usageQuery.data?.user_count ?? 0)}
                <span class="ml-2 text-sm font-medium text-neutral-600">
                  of{" "}
                  {formatNumberWithCommas(
                    subscriptionQuery.data?.plan?.user_count || 0,
                  )}
                </span>
              </div>
              <ProgressBar
                width={"200px"}
                max={subscriptionQuery.data?.plan?.user_count || 0}
                progress={usageQuery.data?.user_count || 0}
              />
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total File Storage</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(usageQuery.data?.file_storage ?? 0)} mb
                <span class="ml-2 text-sm font-medium text-neutral-600">
                  of{" "}
                  {formatStorageMb(
                    subscriptionQuery.data?.plan?.file_storage || 0,
                  )}{" "}
                </span>
              </div>
              <ProgressBar
                width={"200px"}
                max={subscriptionQuery.data?.plan?.file_storage || 0}
                progress={usageQuery.data?.file_storage || 0}
              />
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total Message Count</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(usageQuery.data?.message_count ?? 0)}
                <span class="ml-2 text-sm font-medium text-neutral-600">
                  of{" "}
                  {formatNumberWithCommas(
                    subscriptionQuery.data?.plan?.message_count ?? 0,
                  )}
                </span>
              </div>
              <ProgressBar
                width={"200px"}
                max={subscriptionQuery.data?.plan?.message_count || 0}
                progress={usageQuery.data?.message_count || 0}
              />
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total Chunk Count</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(usageQuery.data?.chunk_count ?? 0)}
                <span class="ml-2 text-sm font-medium text-neutral-600">
                  of{" "}
                  {formatNumberWithCommas(
                    subscriptionQuery.data?.plan?.chunk_count ?? 0,
                  )}
                </span>
              </div>
              <ProgressBar
                width={"200px"}
                max={subscriptionQuery.data?.plan?.chunk_count || 0}
                progress={usageQuery.data?.chunk_count || 0}
              />
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6 md:!border-r">
          <dt class="text-base font-normal">Total Dataset Count</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(usageQuery.data?.dataset_count ?? 0)}
                <span class="ml-2 text-sm font-medium text-neutral-600">
                  of{" "}
                  {formatNumberWithCommas(
                    subscriptionQuery.data?.plan?.dataset_count ?? 0,
                  )}
                </span>
              </div>
              <ProgressBar
                width={"200px"}
                max={subscriptionQuery.data?.plan?.dataset_count || 0}
                progress={usageQuery.data?.dataset_count || 0}
              />
            </div>
          </dd>
        </div>
        {/*
        <div class="px-4 py-5 sm:p-6 md:!border-r">
          <dt class="text-base font-normal">Ingested Tokens</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(usageQuery.data?.tokens_ingested ?? 0)}
              </div>
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6 md:!border-r">
          <dt class="text-base font-normal">Bytes Ingested</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatStorageBytes(usageQuery.data?.bytes_ingested ?? 0)}
              </div>
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6 md:!border-r">
          <dt class="text-base font-normal">Search Tokens</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(usageQuery.data?.search_tokens ?? 0)}
              </div>
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6 md:!border-r">
          <dt class="text-base font-normal">Total Message Tokens</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(usageQuery.data?.message_tokens ?? 0)}
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
                  usageQuery.data?.ocr_pages_ingested ?? 0,
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
                  usageQuery.data?.website_pages_scraped ?? 0,
                )}
              </div>
            </div>
          </dd>
        </div>*/}
      </dl>
    </div>
  );
};
