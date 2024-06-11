import { Accessor } from "solid-js";
import {
  OrganizationUsageCount,
  OrganizationAndSubAndPlan,
} from "../types/apiTypes";

export interface OrganizationUsageOverviewProps {
  organization: Accessor<OrganizationAndSubAndPlan | undefined>;
  orgUsage: Accessor<OrganizationUsageCount | undefined>;
}

export const OrganizationUsageOverview = (
  props: OrganizationUsageOverviewProps,
) => {
  return (
    <div class="mb-3 grid grid-cols-1 gap-5 lg:grid-cols-4">
      <dl class="col-span-4 grid grid-cols-1 divide-y divide-gray-200 overflow-hidden rounded-lg border bg-white shadow md:grid-cols-2 md:divide-x md:divide-y-0">
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal"> Total Users </dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex items-baseline text-2xl font-semibold text-magenta">
              {props.orgUsage()?.user_count}
              <span class="ml-2 text-sm font-medium text-neutral-600">
                of {props.organization()?.plan?.user_count}
              </span>
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total File Storage</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex items-baseline text-2xl font-semibold text-magenta">
              {props.orgUsage()?.file_storage} mb
              <span class="ml-2 text-sm font-medium text-neutral-600">
                of {props.organization()?.plan?.file_storage} mb
              </span>
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total Message Count</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex items-baseline text-2xl font-semibold text-magenta">
              {props.orgUsage()?.message_count}
              <span class="ml-2 text-sm font-medium text-neutral-600">
                of {props.organization()?.plan?.message_count}
              </span>
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total Chunk Count</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex items-baseline text-2xl font-semibold text-magenta">
              {props.orgUsage()?.chunk_count}
              <span class="ml-2 text-sm font-medium text-neutral-600">
                of {props.organization()?.plan?.chunk_count}
              </span>
            </div>
          </dd>
        </div>
      </dl>
    </div>
  );
};
