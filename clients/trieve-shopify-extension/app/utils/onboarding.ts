import { FC, ReactNode } from "react";

type OnboardingStep = {
  id: string;
  title: string;
  description: string;
  icon: ReactNode;
  body: FC;
};
