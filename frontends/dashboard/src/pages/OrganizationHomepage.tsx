import { For, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { ApiContext } from "..";
import { createQuery } from "@tanstack/solid-query";
import { Spacer } from "../components/Spacer";
import { A } from "@solidjs/router";
import { FiDatabase, FiSearch } from "solid-icons/fi";

export const OrganizationHomepage = () => {
  const userData = useContext(UserContext);
  return (
    <div class="p-4 px-8">
      <div class="text-2xl font-medium">
        {userData.selectedOrganization().name}
      </div>
      <Spacer h={16} />
      <div class="-pb-2 w-[400px]">Datasets</div>
      <DatasetOverviewGrid />
      <Spacer h={16} />
      <OrgSettingsLinks />
    </div>
  );
};

const OrgSettingsLinks = () => {
  return (
    <div class="grid grid-cols-3 gap-2">
      <div class="">
        <div>Playgrounds</div>
        <div class="flex flex-col gap-2 rounded-md border-b-neutral-200 bg-white p-2 last:border-b-0">
          <A
            class="flex items-center gap-2 border-b border-b-neutral-300 p-2 last:border-b-0"
            href="/search"
          >
            <FiSearch class="text-neutral-500" />
            Search
          </A>
        </div>
      </div>
      <div class="">
        <div>Playgrounds</div>
        <div class="rounded-md bg-white p-2">
          <A href="/search">Search</A>
        </div>
      </div>
    </div>
  );
};

const DatasetOverviewGrid = () => {
  const trieve = useContext(ApiContext);
  const userData = useContext(UserContext);

  const data = createQuery(() => ({
    queryKey: ["org-homepage-datasets"],
    queryFn: async () => {
      return trieve.fetch(
        "/api/dataset/organization/{organization_id}",
        "get",
        {
          organizationId: userData.selectedOrganization().id,
          limit: 30,
        },
      );
    },
  }));

  return (
    <div class="grid grid-cols-3 gap-4">
      {
        <For each={data.data}>
          {(dataset) => (
            <A href={`/dataset/${dataset.dataset.id}`}>
              <div class="rounded-md border border-neutral-200 bg-white p-2 shadow-md">
                <div class="flex items-center gap-2">
                  <FiDatabase class="text-neutral-400" />
                  <div class="">{dataset.dataset.name}</div>
                </div>
              </div>
            </A>
          )}
        </For>
      }
    </div>
  );
};
