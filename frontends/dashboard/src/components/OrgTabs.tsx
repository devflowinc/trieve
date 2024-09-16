import { A } from "@solidjs/router";

export const OrgTabs = () => {
  return (
    <div class="flex gap-4">
      <A
        end={true}
        href={`/org`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Overview
      </A>
      <A
        href={"/org/users"}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Users
      </A>
      <A
        href="/org/billing"
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Billing
      </A>
      <A
        href="/org/keys"
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        API Keys
      </A>
      <A
        href="/org/settings"
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Settings
      </A>
    </div>
  );
};
