import { Banner, Button, Box } from "@shopify/polaris";
import { useOnboarding } from "app/utils/onboarding";

export const Onboarding = () => {
  const onboarding = useOnboarding();

  if (onboarding.currentStep.hidden) {
    return null;
  }

  return (
    <Banner
      title={onboarding.currentStep.title}
      tone={onboarding.stepIsComplete ? "success" : "info"}
    >
      <Box>
        <onboarding.currentStep.body
          goToNextStep={onboarding.goToNextStep}
          goToPreviousStep={onboarding.goToPreviousStep}
          broadcastCompletion={() => {
            onboarding.setStepCompletions((prev) => ({
              ...prev,
              [onboarding.currentStep.id]: true,
            }));
          }}
        />
        <div className="flex w-full justify-end">
          {onboarding.stepIsComplete &&
            !(onboarding.currentStep?.hideNextButton || false) && (
              <Button
                onClick={() => {
                  onboarding.goToNextStep();
                }}
              >
                {onboarding.currentStep.nextButtonText || "Next"}
              </Button>
            )}
        </div>
      </Box>
    </Banner>
  );
};
