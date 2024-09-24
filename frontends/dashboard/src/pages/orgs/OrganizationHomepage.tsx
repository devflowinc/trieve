import { DatasetOverview } from "../../components/DatasetOverview";
import { GettingStartedDocsLinks } from "../../components/GettingStartedDocsLinks";

export const OrganizationHomepage = () => {
  return (
    <div class="pb-8">
      <div class="h-1" />
      <DatasetOverview />
      <div class="h-6" />
      <GettingStartedDocsLinks />
    </div>
  );
};
