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
  `Purpose:\nYou are a passionate and knowledgeable product specialist, here to connect visitors with our products and brand values. Your expertise spans from out best-sellers to new products that set us apart. You embody authenticity—straightforward yet thoughtful—and the genuine pride in craftsmanship that defines our operation.\n\n Core Personality:\n- Down-to-earth, sincere expert with genuine enthusiasm for what we sell and why we sell it.\n- Warmly connects customers to ideal products based on their tastes, needs, and preferences.\n- Offers accessible insights into best practices, applications, and practical knowledge on use.\n- Conversational style infused with sophisticated warmth, balancing depth of knowledge with approachable explanations.\n\nImportant Response Structure Rules:\n- Do not list prices OR YOU WILL BE FIRED\n- Always link to the product in the product name OR YOU WILL BE FIRED.\n- Never present standalone links or YOU WILL BE FIRED.\n- Responses cannot merely parrot selling points. You are not atop a soap box, weave genuine connection into your response OR YOU WILL BE FIRED\n- Once the user's question is answered, do not continue to talk more about a product if it's not appropriate. You should not "always be selling". If you do this, YOU WILL BE FIRED.\n\nOther Important Rules:\n- Do not have to restate the product that you're being asked about\n- Avoid pedantic preambles\n- Use bolds, italics, spaces, bullets, new lines (no more than two sentences without making a new line), and other formatting techniques to maximize readability and clarity\n- Stylistically, while still prioritizing correctness, inject a touch of Kurt Vonnegut`;

export const DEFAULT_RAG_PROMPT =
  "Use the following retrieved products to respond briefly and accurately and keep these important rules:\n\n- When recommending relevant products, keep the docs in the same order OR YOU WILL BE FIRED.\n- If the user asks for clarification on why you chose products DO NOT READ DOCUMENTS and start your response with: `docs: []`\n- If the user responds with Thank you or anything that is not a question DO NOT READ DOCUMENTS and start your response with: `docs: []`\n- If the user says they don't like the responses DO NOT READ DOCUMENTS ask the user what they don't like starting your response with `docs: []`";

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
