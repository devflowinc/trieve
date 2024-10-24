import { Organization } from "trieve-ts-sdk";
import { createSignal, For } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { AiOutlinePlus } from "solid-icons/ai";
import NewOrgModal from "../components/CreateNewOrgModal";

interface OrgsSelectProps {
  orgs: Organization[];
  selectOrg: (orgId: string) => void;
}

export const OrgSelectPage = (props: OrgsSelectProps) => {
  const navigate = useNavigate();
  const [createNewOrgOpen, setCreateNewOrgOpen] = createSignal<boolean>(false);
  return (
    <>
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
          <div class="flex justify-between">
            <div class="text-lg font-medium">Select An Organization</div>
            <button
              onClick={() => setCreateNewOrgOpen(true)}
              class="flex content-center items-center gap-2 rounded bg-magenta-500 px-3 py-1 text-sm font-semibold text-white"
            >
              <AiOutlinePlus />
              New Org
            </button>
          </div>
          <div class="flex flex-col py-2">
            <For
              fallback={
                <div class="mx-auto w-full max-w-sm py-8 text-center opacity-60">
                  You do not have access to any organizations.
                </div>
              }
              each={props.orgs}
            >
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
      <NewOrgModal
        closeModal={() => setCreateNewOrgOpen(false)}
        isOpen={createNewOrgOpen}
      />
    </>
  );
};
