import { Button } from "@shopify/polaris";
import { CheckCircleIcon } from "@shopify/polaris-icons";
import { useMutation, useQuery } from "@tanstack/react-query";
import { ApiType } from "app/api/root";
import { hc } from "hono/client";

export const JudgeMeSync = () => {
  const client = hc<ApiType>("/");

  const { data: reviewCount } = useQuery({
    queryKey: ["judge-me-review-count"],
    queryFn: async () => {
      const response = await client.api.judgeme.reviewCount.$get();
      if (!response.ok) {
        throw new Error("Failed to get review count");
      }
      const data = await response.json();
      return data;
    },
  });

  const { data: syncedCount } = useQuery({
    queryKey: ["judge-me-sync-count"],
    queryFn: async () => {
      const response = await client.api.judgeme.syncedReviewCount.$get();
      if (!response.ok) {
        throw new Error("Failed to get review count");
      }
      const data = await response.json();
      return data.reviewCount;
    },
    refetchInterval: 1000 * 5,
  });

  const runSyncMutation = useMutation({
    mutationKey: ["judge-me-sync"],
    mutationFn: async () => {
      const response = await client.api.judgeme.sync.$post();
      if (!response.ok) {
        throw new Error("Failed to sync judge.me reviews");
      }
      const data = await response.json();
      return data;
    },
  });

  return (
    <div className="flex gap-4 items-center">
      <Button
        onClick={() => {
          runSyncMutation.mutate();
        }}
      >
        Sync Reviews
      </Button>
      <div className="flex gap-2 items-center">
        <div>
          Synced {syncedCount?.toLocaleString()}/
          {reviewCount?.count.toLocaleString()} reviews.
        </div>
        {syncedCount?.toLocaleString() ===
          reviewCount?.count.toLocaleString() && (
          <CheckCircleIcon height={20} width={20} fill="green" />
        )}
      </div>
    </div>
  );
};
