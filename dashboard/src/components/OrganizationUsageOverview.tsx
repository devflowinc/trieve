import { Accessor } from "solid-js";
import {
  OrganizationUsageCount,
  OrganizationAndSubAndPlan,
} from "shared/types";
import { ProgressBar } from "./ProgressBar";
import { formatNumberWithCommas, formatStorage } from "../utils/formatNumbers";

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
              {formatNumberWithCommas(props.orgUsage()?.user_count ?? 0)}
              <span class="ml-2 text-sm font-medium text-neutral-600">
                of{" "}
                {formatNumberWithCommas(
                  props.organization()?.plan?.user_count ?? 0,
                )}
              </span>
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total File Storage</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(props.orgUsage()?.file_storage ?? 0)} mb
                <span class="ml-2 text-sm font-medium text-neutral-600">
                  of{" "}
                  {formatStorage(props.organization()?.plan?.file_storage ?? 0)}{" "}
                </span>
              </div>
              <ProgressBar
                width={"200px"}
                max={props.organization()?.plan?.file_storage ?? 0}
                progress={props.orgUsage()?.file_storage ?? 0}
              />
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total Message Count</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(props.orgUsage()?.message_count ?? 0)}
                <span class="ml-2 text-sm font-medium text-neutral-600">
                  of{" "}
                  {formatNumberWithCommas(
                    props.organization()?.plan?.message_count ?? 0,
                  )}
                </span>
              </div>
              <ProgressBar
                width={"200px"}
                max={props.organization()?.plan?.message_count ?? 0}
                progress={props.orgUsage()?.message_count ?? 0}
              />
            </div>
          </dd>
        </div>
        <div class="px-4 py-5 sm:p-6">
          <dt class="text-base font-normal">Total Chunk Count</dt>
          <dd class="mt-1 flex items-baseline justify-between md:block lg:flex">
            <div class="flex flex-col items-baseline gap-3 text-2xl font-semibold text-magenta">
              <div>
                {formatNumberWithCommas(props.orgUsage()?.chunk_count ?? 0)}
                <span class="ml-2 text-sm font-medium text-neutral-600">
                  of{" "}
                  {formatNumberWithCommas(
                    props.organization()?.plan?.chunk_count ?? 0,
                  )}
                </span>
              </div>
              <ProgressBar
                width={"200px"}
                max={props.organization()?.plan?.chunk_count ?? 0}
                progress={props.orgUsage()?.chunk_count ?? 0}
              />
            </div>
          </dd>
        </div>
      </dl>
    </div>
  );
};
