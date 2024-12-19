import { createContext, ReactNode, useEffect, useState } from "react";
import React from "react";
import { useModalState } from "../../utils/hooks/modal-context";

const FileContext = createContext<{
  files: Record<string, string>;
}>({ files: {} });

export const FileContextProvider = (props: { children: ReactNode }) => {
  const [files, setFiles] = useState<Record<string, string>>({});
  const [page] = useState(1);
  const state = useModalState();

  useEffect(() => {
    const getFiles = async () => {
      const files = state.trieveSDK.trieve.fetch(
        "/api/dataset/files/{dataset_id}/{page}",
        "get",
        {
          page: 1,
          datasetId: state.props.datasetId,
        },
      );
    };
    getFiles();
  }, []);

  return (
    <FileContext.Provider value={{ files: files }}>
      {props.children}
    </FileContext.Provider>
  );
};
