import { useQuery } from "@tanstack/react-query";
import { ApiType } from "app/api/root";
import { hc } from "hono/client";

export const JudgeMeSync = () => {
  const client = hc<ApiType>("/");

  const { data: reviewCount } = useQuery({
    queryKey: ["judge-me-sync-count"],
    queryFn: async () => {
      const response = await client.api.judgeme.reviewCount.$get();
      if (!response.ok) {
        throw new Error("Failed to get review count");
      }
      const data = await response.json();
      return data;
    },
  });

  return (
    <div>
      <div>JudgeMeSync</div>
      <div>{JSON.stringify(reviewCount?.count)}</div>
    </div>
  );
};
