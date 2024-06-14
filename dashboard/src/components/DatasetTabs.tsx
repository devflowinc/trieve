import { A, useParams } from "@solidjs/router";

export const DatasetTabs = () => {
  const urlParams = useParams();

  return (
    <div class="flex space-x-4">
      <A
        href={`/dashboard/dataset/${urlParams.id}/start`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Start
      </A>
      <A
        href={`/dashboard/dataset/${urlParams.id}/events`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Events
      </A>
      <A
        href={`/dashboard/dataset/${urlParams.id}/api-keys`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        API Keys
      </A>
      <A
        href={`/dashboard/dataset/${urlParams.id}/settings`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Settings
      </A>
    </div>
  );
};
