import { Banner, Button, Box, Card, Text } from "@shopify/polaris";
import { useOnboarding } from "app/utils/onboarding";

export const Onboarding = () => {
  const onboarding = useOnboarding();

  if (onboarding.currentStep.hidden) {
    return null;
  }

  return (
    <Card>
      <div className="flex justify-between">
        <Text variant="headingMd" as="h2">
          Getting Started
        </Text>
        <div className="flex gap-4">
          <button onClick={onboarding.goToPreviousStep}>Prev</button>
          <button onClick={onboarding.goToNextStep}>Next</button>
        </div>
      </div>
      <div>
        {onboarding.allSteps.map((step) => {
          return (
            <div className="border-t border-t-neutral-200">
              <div className="flex p-2">
                <div>{step.title}</div>
              </div>
              <div
                style={{
                  height: onboarding.currentStep.id === step.id ? 100 : 0,
                }}
                className="transition-all overflow-hidden"
              >
                {
                  <step.body
                    goToNextStep={onboarding.goToNextStep}
                    goToPreviousStep={onboarding.goToPreviousStep}
                    broadcastCompletion={() => {
                      onboarding.setStepCompletions((prev) => ({
                        ...prev,
                        [onboarding.currentStep.id]: true,
                      }));
                    }}
                  />
                }
              </div>
            </div>
          );
        })}
      </div>
    </Card>
  );
};
