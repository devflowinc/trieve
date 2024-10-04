import {
  createQuery,
  CreateQueryOptions,
  UndefinedInitialDataOptions,
} from "@tanstack/solid-query";
import { useTrieve } from "../hooks/useTrieve";
import { useContext } from "solid-js";
import { DatasetContext } from "../contexts/DatasetContext";
import { UserContext } from "../contexts/UserContext";
import { TrieveFetchClient } from "trieve-ts-sdk";

type queryFnCtx = {
  datasetId: string;
  orgId: string;
  trieve: TrieveFetchClient;
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
interface createTrieveQueryArgs<T, D extends Record<string, any>> {
  queryFn: (ctx: queryFnCtx & D) => Promise<T>;
  deps?: D;
}
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const createTrieveQuery = <T, D extends Record<string, any>>(
  name: string,
  args: createTrieveQueryArgs<T, D>,
  queryOptions?: CreateQueryOptions<any>,
) => {
  const trieve = useTrieve();
  const { datasetId } = useContext(DatasetContext);
  const userContext = useContext(UserContext);

  const result = createQuery(() => ({
    queryKey: [
      name,
      {
        datasetId: datasetId(),
        orgId: userContext.selectedOrg().id,
      },
    ],
    queryFn: async () => {
      // Build context
      const context = {
        get datasetId() {
          return datasetId();
        },
        get orgId() {
          return userContext.selectedOrg().id;
        },
        trieve,
      };
      // Call queryFn with context
      return await args.queryFn({
        ...context,
        ...(args.deps as D),
      });
    },
    ...queryOptions,
  }));

  return result;
};
