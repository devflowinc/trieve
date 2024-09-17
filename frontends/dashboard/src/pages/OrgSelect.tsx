import { Organization } from "trieve-ts-sdk";
import { For } from "solid-js";
import { useNavigate } from "@solidjs/router";

interface OrgsSelectProps {
  orgs: Organization[];
  selectOrg: (orgId: string) => void;
}

export const OrgSelectPage = (props: OrgsSelectProps) => {
  const navigate = useNavigate();
  return (
    <div class="relative flex min-h-screen flex-col items-center bg-neutral-200 py-36">
      <div class="absolute left-4 top-2 mb-8 flex items-center gap-1">
        <img
          class="h-12 w-12 cursor-pointer"
          src="https://cdn.trieve.ai/trieve-logo.png"
          alt="Logo"
        />
        <span class="text-2xl font-semibold">Trieve</span>
      </div>
      <div class="rounded-md border border-neutral-300 bg-white p-4 md:min-w-[500px]">
        <div class="text-lg font-medium">Select An Organization</div>
        <div class="flex flex-col py-2">
          <For each={props.orgs}>
            {(org) => (
              <button
                onClick={() => {
                  props.selectOrg(org.id);
                  navigate("/org");
                }}
                class="flex cursor-pointer items-center justify-between rounded-md border-b border-b-neutral-200 p-2 last:border-b-transparent hover:bg-neutral-100"
              >
                <div class="flex w-full items-center justify-between">
                  <div class="text-sm font-medium">{org.name}</div>
                  <div class="text-xs text-neutral-500">{org.id}</div>
                </div>
              </button>
            )}
          </For>
        </div>
      </div>
    </div>
  );
};
