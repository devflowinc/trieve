import { useMatches } from "@remix-run/react";
import merge from "deepmerge";
import { DehydratedState } from "@tanstack/react-query";

export const useDehydratedState = (): DehydratedState | undefined => {
  const matches = useMatches();

  const dehydratedState = matches
    // @ts-ignore
    .map((match) => match.data?.dehydratedState)
    .filter(Boolean);

  return dehydratedState.length
    ? dehydratedState.reduce(
        (accumulator, currentValue) => merge(accumulator, currentValue),
        {},
      )
    : // @ts-ignore
      undefined;
};
