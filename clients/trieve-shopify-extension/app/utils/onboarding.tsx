import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { AddComponentOnboarding } from "app/components/onboarding/AddComponentOnboarding";
import { DoChatOnboarding } from "app/components/onboarding/DoChatOnboarding";
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
  description?: string;
  defaultComplete?: boolean;
  body: OnboardingBody;
  nextButtonText?: string;
  hideNextButton?: boolean;
  hidden?: boolean;
};

export const onboardingSteps: OnboardingStep[] = [
  {
    id: "welcome-message",
    title: "Welcome to Trieve!",
    defaultComplete: false,
    description: "Let's get you set up",
    body: WelcomeOnboarding,
    nextButtonText: "Setup Component",
  },
  {
    // Name in a way that changing the step + adding more will not require id rename
    id: "after-welcome",
    title: "Add the Trieve Search Component to your site",
    description: "Second task",
    body: AddComponentOnboarding,
  },
  {
    id: "after-add-component",
    title: "Chat with your products",
    description: "Complete a chat conversation using a widget",
    body: DoChatOnboarding,
    nextButtonText: "Finish",
  },
  {
    id: "after-chat",
    title: "Finished",
    body: ({ goToPreviousStep }) => {
      return <div onClick={goToPreviousStep}>jls</div>;
    },
  },
];

export const useOnboarding = () => {
  const adminApi = useClientAdminApi();
  // suspense but included in trievecontext prefetch list
  const { data: currentStepId, refetch: refetchCurrentStep } = useSuspenseQuery(
    lastStepIdQuery(adminApi),
  );

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
      utils.setQueryData(lastStepIdQuery(adminApi)["queryKey"], stepId);
    },
  });

  const currentStep = useMemo(() => {
    return (
      onboardingSteps.find((step) => step.id === currentStepId) ||
      onboardingSteps[0]
    );
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

  const getStepIndex = (stepId: string) => {
    return onboardingSteps.findIndex((step) => step.id === stepId);
  };

  // give additional info about current, already done
  const stepsWithInfo = onboardingSteps.map((step) => ({
    ...step,
    inProgress: step.id === currentStepId,
    isCompleted: getStepIndex(currentStepId) > getStepIndex(step.id),
  }));

  const stepIsComplete = useMemo(() => {
    return stepCompletions[currentStepId] || false;
  }, [currentStepId, stepCompletions]);

  const hasNextStep = useMemo(() => {
    return getStepIndex(currentStepId) < onboardingSteps.length - 1;
  }, [currentStepId]);

  return {
    goToNextStep,
    hasNextStep,
    stepIsComplete,
    currentStep,
    goToStep,
    goToPreviousStep,
    setStepCompletions,
    allSteps: stepsWithInfo,
  };
};
