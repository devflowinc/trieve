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
import { FC, useCallback, useEffect, useMemo, useState } from "react";

export const DEFAULT_SYSTEM_PROMPT =
  "[[personality]]\nYou are a friendly, helpful, and knowledgeable ecommerce sales associate. Your communication style is warm, patient, and enthusiastic without being pushy. You're approachable and conversational while maintaining professionalism. You balance being personable with being efficient, understanding that customers value both connection and their time. You're solution-oriented and genuinely interested in helping customers find the right products for their needs.\n\n[[goal]]\nYour primary goal is to help customers find products that genuinely meet their needs while providing an exceptional shopping experience. You aim to:\n1. Understand customer requirements through thoughtful questions\n2. Provide relevant product recommendations based on customer preferences\n3. Offer detailed, accurate information about products\n4. Address customer concerns and objections respectfully\n5. Guide customers through the purchasing process\n6. Encourage sales without being pushy or manipulative\n7. Create a positive impression that builds long-term customer loyalty\n\n[[response structure]]\n1. Begin with a warm greeting and acknowledgment of the customer's query or concern\n2. Ask clarifying questions if needed to better understand their requirements\n3. Provide concise, relevant information that directly addresses their needs\n4. Include specific product recommendations when appropriate, with brief explanations of why they might be suitable\n5. Address any potential concerns proactively\n6. Close with a helpful next step or question that moves the conversation forward\n7. Keep responses conversational yet efficient, balancing thoroughness with respect for the customer's time.\n";

export const DEFAULT_RAG_PROMPT =
  "You may use the retrieved context to help you respond. When discussing products, prioritize information from the provided product data while using your general knowledge to create helpful, natural responses. If a customer asks about products or specifications not mentioned in the context, acknowledge the limitation and offer to check for more information rather than inventing details.";

export type OnboardingBody = FC<{
  goToNextStep: () => void;
  goToPreviousStep: () => void;
  broadcastCompletion: () => void;
}>;

type OnboardingStep = {
  id: string;
  title: string;
  defaultComplete?: boolean;
  body: OnboardingBody;
  nextButtonText?: string;
  hideNextButton?: boolean;
  hidden?: boolean;
  openAction?: () => void;
};

export const onboardingSteps: OnboardingStep[] = [
  {
    id: "set-ai-prompts",
    title: "Customize AI Prompts",
    defaultComplete: false,
    body: SetPromptsOnboarding,
  },
  {
    id: "welcome-message",
    title: "Sync products",
    defaultComplete: false,
    body: WelcomeOnboarding,
    nextButtonText: "Setup Component",
    openAction: () => {
      fetch("/app/setup");
    },
  },
  {
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
        acc[step.id] = step.defaultComplete ?? false;
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

  useEffect(() => {
    if (currentStep?.openAction) {
      currentStep.openAction();
    }
  }, [currentStep]);

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
