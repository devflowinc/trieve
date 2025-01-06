import { DatasetOverview } from "../../components/DatasetOverview";
import { GettingStartedDocsLinks } from "../../components/GettingStartedDocsLinks";
import { OnboardingSteps } from "../../components/OnboardingSteps";
import OrgUpdateAlert from "../../components/OrgUpdateAlert";

export const OrganizationHomepage = () => {
  return (
    <div class="pb-8">
      <OrgUpdateAlert />
      <div class="h-1" />
      <OnboardingSteps />
      <div class="h-1" />
      <DatasetOverview />
      <div class="h-6" />
      <GettingStartedDocsLinks />
    </div>
  );
};
