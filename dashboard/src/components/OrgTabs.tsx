import { A } from "@solidjs/router";

export const OrgTabs = () => {
  return (
    <div class="flex space-x-4">
      <A
        href="/dashboard/overview"
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Overview
      </A>
      <A
        href="/dashboard/users"
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Users
      </A>
      <A
        href="/dashboard/billing"
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Billing
      </A>
      <A
        href="/dashboard/settings"
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Settings
      </A>
    </div>
  );
};
