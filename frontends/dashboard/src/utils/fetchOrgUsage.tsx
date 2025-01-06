import { createQuery } from "@tanstack/solid-query";
import type { UserStore } from "../contexts/UserContext";
import type { OrganizationAndSubAndPlan } from "shared/types";
import { TrieveFetchClient } from "trieve-ts-sdk";

export const createUsageQuery = (
  userContext: UserStore,
  trieve: TrieveFetchClient,
) =>
  createQuery(() => ({
    queryKey: ["org-usage", userContext.selectedOrg().id],
    queryFn: async () => {
      return trieve.fetch("/api/organization/usage/{organization_id}", "get", {
        organizationId: userContext.selectedOrg().id,
      });
    },
  }));

export const createSubscriptionQuery = (
  userContext: UserStore,
  trieve: TrieveFetchClient,
) =>
  createQuery(() => ({
    queryKey: ["org-subscription", userContext.selectedOrg().id],
    queryFn: async () => {
      return trieve.fetch<"eject">(
        "/api/organization/{organization_id}",
        "get",
        {
          organizationId: userContext.selectedOrg().id,
        },
      ) as Promise<OrganizationAndSubAndPlan>;
    },
  }));
