import React, { useRef, useEffect, useState } from "react";

import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import { TrieveModalSearch } from "trieve-search-component";
import "trieve-search-component/dist/index.css";
import { usePluginData } from "@docusaurus/useGlobalData";

const SearchComponent = () => {
  const { options } = usePluginData("docusaurus-trieve-search-theme");

  const htmlRef = useRef(null);
  const [theme, setTheme] = useState("light");
  const observerRef = useRef(null);


  useEffect(() => {
    htmlRef.current = document.querySelector("html");
  });

  useEffect(() => {

    if (htmlRef && htmlRef.current) {
      getThemeMode();

      observerRef.current = new MutationObserver(onElementUpdate);
      observerRef.current.observe(htmlRef.current, { attributes: true });
    }

    return () => {
      observerRef.current.disconnect();
    }
  }, [htmlRef]);

  // Callback function to execute when mutations are observed
  function onElementUpdate(mutationsList, observer) {
      for(let mutation of mutationsList) {
          if (mutation.type === 'attributes') {
              getThemeMode();
          }
      }
  };

  function getThemeMode() {
    if (htmlRef && htmlRef.current) {
      setTheme(htmlRef.current.getAttribute("data-theme"))
    }
  }

  return <TrieveModalSearch {...options} theme={theme} />;
};

export default SearchComponent;
