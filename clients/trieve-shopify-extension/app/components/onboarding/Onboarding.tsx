import { Card, Collapsible, Text } from "@shopify/polaris";
import { CaretUpIcon } from "@shopify/polaris-icons";
import { cn } from "app/utils/cn";
import { useOnboarding } from "app/utils/onboarding";

export const Onboarding = () => {
  const onboarding = useOnboarding();

  if (onboarding.currentStep.hidden) {
    return null;
  }

  return (
    <Card padding={"0"}>
      <div className="flex justify-between p-4">
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
          const isCurrent = onboarding.currentStep.id === step.id;
          return (
            <div className="border-t px-2 border-t-neutral-200">
              <button
                onClick={() => {
                  onboarding.goToStep(step.id);
                }}
                className="flex w-full justify-between items-center p-1"
              >
                <div
                  className={cn(isCurrent ? "text-black" : "text-neutral-400")}
                >
                  {step.title}
                </div>
                <div
                  className={cn(
                    "p-2 fill-black",
                    isCurrent ? "rotate-180" : "",
                  )}
                >
                  <CaretUpIcon width={20} height={20} />
                </div>
              </button>
              <Collapsible open={isCurrent} id={step.id}>
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
              </Collapsible>
            </div>
          );
        })}
      </div>
    </Card>
  );
};
