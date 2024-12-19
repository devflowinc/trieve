import { createContext, ReactNode, useEffect, useState } from "react";
import React from "react";
import { useModalState } from "../../utils/hooks/modal-context";

const FileContext = createContext<{
  files: Record<string, string>;
}>({ files: {} });

export const FileContextProvider = (props: { children: ReactNode }) => {
  const [files, setFiles] = useState<Record<string, string>>({});
  const state = useModalState();

  useEffect(() => {
    const getFiles = async () => {
      const page = 1;
      let done = false;
      const fileMapResult: Record<string, string> = {};
      while (!done) {
        const files = await state.trieveSDK.trieve.fetch(
          "/api/dataset/files/{dataset_id}/{page}",
          "get",
          {
            page,
            datasetId: state.props.datasetId,
          },
        );

        if (files.length) {
          files.reduce((acc, file) => {
            acc[file.file_name] = file.id;
            return acc;
          }, fileMapResult);
        } else {
          done = true;
        }
      }

      setFiles(fileMapResult);
    };
    void getFiles();
  }, []);

  return (
    <FileContext.Provider value={{ files: files }}>
      {props.children}
    </FileContext.Provider>
  );
};
