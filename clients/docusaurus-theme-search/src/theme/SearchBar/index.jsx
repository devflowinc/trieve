import React from "react";

import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import { TrieveModalSearch } from "trieve-search-component";
import "trieve-search-component/dist/index.css";
import { usePluginData } from "@docusaurus/useGlobalData";

const SearchComponent = () => {
  const { options } = usePluginData("docusaurus-trieve-search-theme");

  return <TrieveModalSearch {...options} />;
};

export default SearchComponent;
