import { KnownEventNames } from "app/utils/formatting";

export const chatEvents: KnownEventNames[] = [
  "component_load",
  "component_open",
  "conversation_started",
  "site-add_to_cart",
  "site-checkout",
];

export const searchEvents: KnownEventNames[] = [
  "component_load",
  "component_open",
  "searched",
  "site-add_to_cart",
  "site-checkout",
];

export const recommendationEvents: KnownEventNames[] = [
  "component_load",
  "component_open",
  "recommendation_created",
  "site-add_to_cart",
  "site-checkout",
];