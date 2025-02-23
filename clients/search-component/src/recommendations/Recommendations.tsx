import React from "react";
import { useRecommendations } from "./use-recommendations";
import { cva } from "cva";
export interface Theme {
  mode: "light" | "dark";
  rounded: "sm" | "md" | "lg" | "none";
  font: string;
  padding: "sm" | "md" | "lg" | "xl";
}

export interface RecommendationsConfig {
  theme?: Theme;
  datasetId: string;
  productId: string;
  apiKey: string;
  baseUrl?: string;
  maxResults?: number;
  title?: string;
  orientation?: "horizontal" | "vertical";
}

const outerClass = cva(["tv-flex", "tv-gap-2"], {
  variants: {
    orientation: {
      horizontal: "tv-flex-row",
      vertical: "tv-flex-col",
    },
  },
  defaultVariants: {
    orientation: "horizontal",
  },
});

const itemClass = cva([""], {
  variants: {
    orientation: {
      horizontal: "tv-flex-row",
      vertical: "tv-flex-col",
    },
  },
  defaultVariants: {
    orientation: "horizontal",
  },
});

export const Recommendations = (config: RecommendationsConfig) => {
  const { status, results } = useRecommendations(config);

  return (
    <>
      <div
        className={outerClass({
          orientation: config.orientation,
        })}
        style={{
          fontFamily: config.theme?.font || "inherit",
        }}
      >
        {results?.map((r) => (
          <div className={itemClass({ orientation: config.orientation })}>
            <img className="tv-max-w-[100px]" src={r.metadata.images[0].src} />
            <div>{r.metadata.title}</div>
          </div>
        ))}
      </div>
      <div>
        <pre>{JSON.stringify(results, null, 2)}</pre>
      </div>
    </>
  );
};
