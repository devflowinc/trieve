import { Banner, Button, Box } from "@shopify/polaris";
import { useOnboarding } from "app/utils/onboarding";

export const Onboarding = () => {
  const onboarding = useOnboarding();

  return (
    <Banner
      title={onboarding.currentStep.title}
      tone={onboarding.stepIsComplete ? "success" : "info"}
    >
      <Box>
        {onboarding.currentStep?.body({
          goToNextStep: onboarding.goToNextStep,
          goToPreviousStep: onboarding.goToPreviousStep,
        })}
        <div className="flex w-full justify-end">
          {onboarding.stepIsComplete && (
            <Button
              onClick={() => {
                onboarding.goToNextStep();
              }}
            >
              Next
            </Button>
          )}
        </div>
      </Box>
    </Banner>
  );
};
