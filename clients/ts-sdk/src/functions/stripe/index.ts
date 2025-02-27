/**
 * This includes all the functions you can use to communicate with our stripe endpoint
 *
 * @module Stripe Methods
 *
 */

import { TrieveSDK } from "../../sdk";
import { StripePlan } from "../../types.gen";

export async function getStripePlans(
  /** @hidden */
  this: TrieveSDK
): Promise<StripePlan[] | null> {
  if (!this.organizationId) {
    throw new Error("Organization ID is required to get Stripe Customer");
  }
  return this.trieve.fetch("/api/stripe/plans", "get");
}

export async function startStripeCheckout(
  /** @hidden */
  this: TrieveSDK,
  planId: string
): Promise<string> {
  if (!this.organizationId) {
    throw new Error("Organization ID is required to start Stripe Checkout");
  }
  return this.trieve.fetch(
    "/api/stripe/payment_link/{plan_id}/{organization_id}",
    "get",
    {
      planId,
      organizationId: this.organizationId,
    }
  );
}
