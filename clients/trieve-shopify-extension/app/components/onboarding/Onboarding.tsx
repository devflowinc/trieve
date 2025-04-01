import { Card, Collapsible, Text } from "@shopify/polaris";
import { CaretUpIcon, CheckCircleIcon } from "@shopify/polaris-icons";
import { cn } from "app/utils/cn";
import { useOnboarding } from "app/utils/onboarding";
import { is } from "date-fns/locale";
import { useState } from "react";

export const Onboarding = () => {
  const onboarding = useOnboarding();

  if (onboarding.currentStep.hidden) {
    return null;
  }

  return (
    <Card padding={"0"}>
      <div className="flex justify-between p-4">
        <Text variant="headingMd" as="h2">
          Getting Started With Trieve
        </Text>
        <div className="flex gap-4">
          <button
            className="text-[12px] hover:underline opacity-40"
            onClick={onboarding.skipOnboarding}
          >
            Hide Tutorial
          </button>
        </div>
      </div>
      <div>
        {onboarding.allSteps.map((step) => {
          if (step.hidden) {
            return null;
          }
          const isCurrent = onboarding.currentStep.id === step.id;
          const [open, setOpen] = useState(isCurrent);
          const isCompleted = onboarding.stepCompletions[step.id];
          return (
            <div className="border-t px-2 border-t-neutral-200" key={step.id}>
              <button
                onClick={() => {
                  onboarding.goToStep(step.id);
                  setOpen((prev) => !prev);
                }}
                className="flex w-full justify-between items-center p-1 px-2"
              >
                <div className="flex gap-2">
                  <div
                    className={cn(
                      "transition-colors duration-100",
                      isCurrent ? "text-black" : "text-neutral-400",
                    )}
                  >
                    {step.title}
                  </div>
                  {isCompleted && (
                    <CheckCircleIcon height={20} width={20} fill="green" />
                  )}
                </div>
                <div
                  className={cn(
                    "p-2 fill-black transition-transform",
                    open ? "rotate-180" : "",
                  )}
                >
                  <CaretUpIcon width={20} height={20} />
                </div>
              </button>
              <Collapsible expandOnPrint open={open} id={step.id}>
                {
                  <div className="w-full">
                    <step.body
                      goToNextStep={onboarding.goToNextStep}
                      goToPreviousStep={onboarding.goToPreviousStep}
                      broadcastCompletion={() => {
                        onboarding.setStepCompletions((prev) => ({
                          ...prev,
                          [step.id]: true,
                        }));
                      }}
                    />
                  </div>
                }
              </Collapsible>
            </div>
          );
        })}
      </div>
    </Card>
  );
};
