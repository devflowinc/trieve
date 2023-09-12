import { Topic } from "./topics";

export interface ActixApiDefaultError {
  message: string;
}

export interface StripeCheckoutSessionResponse {
  checkout_session_url: string;
}

export const isActixApiDefaultError = (
  data: unknown,
): data is ActixApiDefaultError => {
  return (
    typeof data === "object" &&
    data !== null &&
    "message" in data &&
    typeof (data as ActixApiDefaultError).message === "string"
  );
};

export const isTopic = (data: unknown): data is Topic => {
  return (
    typeof data === "object" &&
    data !== null &&
    "resolution" in data &&
    "side" in data &&
    "id" in data &&
    typeof (data as Topic).resolution === "string" &&
    typeof (data as Topic).side === "boolean" &&
    typeof (data as Topic).id === "string" &&
    (typeof (data as Topic).normal_chat === "undefined" ||
      typeof (data as Topic).normal_chat === "boolean")
  );
};

export const detectReferralToken = (queryParamT: string | undefined) => {
  if (queryParamT) {
    let previousTokens: string[] = [];
    const previousReferralToken = window.localStorage.getItem("referralToken");
    if (previousReferralToken) {
      const previousReferralTokenArray: string[] = JSON.parse(
        previousReferralToken,
      ) as unknown as string[];
      previousTokens = previousReferralTokenArray;
      if (previousTokens.find((val) => val === queryParamT)) {
        return;
      }
    }
    previousTokens.push(queryParamT);
    window.localStorage.setItem(
      "referralToken",
      JSON.stringify(previousTokens),
    );
  }
};

export const getReferralTokenArray = (): string[] => {
  const previousReferralToken = window.localStorage.getItem("referralToken");
  if (previousReferralToken) {
    const previousReferralTokenArray: string[] = JSON.parse(
      previousReferralToken,
    ) as unknown as string[];
    return previousReferralTokenArray;
  }
  return [];
};

export const isStripeCheckoutSessionResponse = (
  data: unknown,
): data is StripeCheckoutSessionResponse => {
  if (
    typeof data === "object" &&
    data !== null &&
    "checkout_session_url" in data &&
    typeof (data as StripeCheckoutSessionResponse).checkout_session_url ===
      "string"
  ) {
    return true;
  }
  return false;
};

export interface UserPlan {
  id: string;
  stripe_customer_id: string;
  stripe_subscription_id: string;
  plan: "silver" | "gold";
  status: string;
  created_at: string;
  updated_at: string;
}

export const isUserPlan = (data: unknown): data is UserPlan => {
  if (
    typeof data === "object" &&
    data !== null &&
    "id" in data &&
    "stripe_customer_id" in data &&
    "stripe_subscription_id" in data &&
    "plan" in data &&
    "created_at" in data &&
    "updated_at" in data &&
    typeof (data as UserPlan).id === "string" &&
    typeof (data as UserPlan).stripe_customer_id === "string" &&
    typeof (data as UserPlan).stripe_subscription_id === "string" &&
    typeof (data as UserPlan).status === "string" &&
    typeof (data as UserPlan).plan === "string" &&
    typeof (data as UserPlan).created_at === "string" &&
    typeof (data as UserPlan).updated_at === "string"
  ) {
    return true;
  }
  return false;
};
