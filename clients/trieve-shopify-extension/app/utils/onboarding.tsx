import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { AddComponentOnboarding } from "app/components/onboarding/AddComponentOnboarding";
import { DoChatOnboarding } from "app/components/onboarding/DoChatOnboarding";
import { SetPromptsOnboarding } from "app/components/onboarding/SetPromptsOnboarding";
import { WelcomeOnboarding } from "app/components/onboarding/WelcomeOnboarding";
import { setMetafield } from "app/loaders";
import { useClientAdminApi } from "app/loaders/clientLoader";
import {
  lastStepIdQuery,
  ONBOARD_STEP_META_FIELD,
} from "app/queries/onboarding";
import { FC, useCallback, useMemo, useState } from "react";

export type OnboardingBody = FC<{
  goToNextStep?: () => void;
  goToPreviousStep?: () => void;
  broadcastCompletion?: () => void;
}>;

type OnboardingStep = {
  id: string;
  title: string;
  defaultComplete?: boolean;
  body: OnboardingBody;
  nextButtonText?: string;
  hideNextButton?: boolean;
  hidden?: boolean;
};

export const onboardingSteps: OnboardingStep[] = [
  {
    id: "set-chat-messages",
    title: "Set AI Prompts",
    defaultComplete: false,
    body: SetPromptsOnboarding,
    nextButtonText: "Setup Component",
  },
  {
    id: "welcome-message",
    title: "Sync products",
    defaultComplete: false,
    body: WelcomeOnboarding,
    nextButtonText: "Setup Component",
  },
  {
    // Name in a way that changing the step + adding more will not require id rename
    id: "after-welcome",
    title: "Add the widgets to your site",
    body: AddComponentOnboarding,
  },
  {
    id: "after-add-component",
    title: "Chat with your products",
    body: DoChatOnboarding,
    nextButtonText: "Finish",
  },
  {
    id: "after-chat",
    title: "Finished",
    hidden: true,
    body: ({ goToPreviousStep }) => {
      return <div onClick={goToPreviousStep}>back</div>;
    },
  },
];

export const useOnboarding = () => {
  const adminApi = useClientAdminApi();
  // suspense but included in trievecontext prefetch list
  const { data: currentStepId } = useSuspenseQuery(lastStepIdQuery(adminApi));

  const [stepCompletions, setStepCompletions] = useState<
    Record<string, boolean>
  >(
    onboardingSteps.reduce(
      (acc, step) => {
        acc[step.id] = step.defaultComplete || false;
        return acc;
      },
      {} as Record<string, boolean>,
    ),
  );

  const utils = useQueryClient();

  const nextStepMutation = useMutation({
    mutationKey: ["next_step"],
    mutationFn: async ({ stepId }: { stepId: string }) => {
      setMetafield(adminApi, ONBOARD_STEP_META_FIELD, stepId);
    },
    onMutate({ stepId }) {
      utils.setQueryData(
        lastStepIdQuery(adminApi)["queryKey"],
        stepId === "null" ? null : stepId,
      );
    },
  });

  const currentStep = useMemo(() => {
    return onboardingSteps.find((step) => step.id === currentStepId) || null;
  }, [currentStepId]);

  const goToNextStep = useCallback(() => {
    let currentStepIndex = onboardingSteps.findIndex(
      (step) => step.id === currentStepId,
    );
    if (currentStepIndex == onboardingSteps.length - 1) {
      // TODO: Finish message
      return;
    } else if (currentStepIndex === -1) {
      currentStepIndex = 0;
    }
    const nextStepId = onboardingSteps[currentStepIndex + 1].id;
    nextStepMutation.mutate({ stepId: nextStepId });
  }, [currentStepId, nextStepMutation]);

  const goToStep = useCallback(
    (stepId: string) => {
      nextStepMutation.mutate({ stepId });
    },
    [nextStepMutation],
  );

  const collapseAllSteps = () => {
    nextStepMutation.mutate({ stepId: "null" });
  };

  const goToPreviousStep = useCallback(() => {
    let currentStepIndex = onboardingSteps.findIndex(
      (step) => step.id === currentStepId,
    );
    if (currentStepIndex === 0) {
      return;
    }
    const previousStepId = onboardingSteps[currentStepIndex - 1].id;
    nextStepMutation.mutate({ stepId: previousStepId });
  }, [currentStepId, nextStepMutation]);

  const skipOnboarding = () => {
    const hiddenSteps = onboardingSteps.filter((step) => step.hidden);
    if (hiddenSteps.length === 0) {
      return;
    }
    goToStep(hiddenSteps[0].id);
  };

  return {
    goToNextStep,
    currentStep,
    goToStep,
    skipOnboarding,
    goToPreviousStep,
    setStepCompletions,
    collapseAllSteps,
    stepCompletions,
    allSteps: onboardingSteps,
  };
};
