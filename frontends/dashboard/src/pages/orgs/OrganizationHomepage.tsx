import { DatasetOverview } from "../../components/DatasetOverview";
import { GettingStartedDocsLinks } from "../../components/GettingStartedDocsLinks";
import { OnboardingSteps } from "../../components/OnboardingSteps";

export const OrganizationHomepage = () => {
  return (
    <div class="pb-8">
      <OnboardingSteps />
      <div class="h-1" />
      <DatasetOverview />
      <div class="h-6" />
      <GettingStartedDocsLinks />
    </div>
  );
};
