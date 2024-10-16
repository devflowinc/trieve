import path from "path";

/**
 * @typedef {import("@docusaurus/types").Plugin} Plugin
 * @typedef {import("@docusaurus/types").DocusaurusContext} DocusaurusContext
 * @typedef {import("webpack").Configuration} WebpackConfiguration
 * @param {DocusaurusContext} context
 */
export default function plugin(context, options) {
  /**
   * @type {Plugin}
   */
  const config = {
    name: "docusaurus-trieve-search-theme",
    async contentLoaded({ actions }) {
      actions.setGlobalData({ options });
    },
  };

  return {
    ...config,
    getThemePath() {
      return path.resolve(__dirname, "./theme");
    },
  };
}
