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
      let page = 1;
      let done = false;
      const fileMapResult: Record<string, string> = {};
      let totalPages = Number.MAX_SAFE_INTEGER;
      while (!done && page <= totalPages) {
        const files = await state.trieveSDK.trieve.fetch(
          "/api/dataset/files/{dataset_id}/{page}",
          "get",
          {
            page,
            datasetId: state.props.datasetId,
          },
        );

        totalPages = files.total_pages;

        if (files.file_and_group_ids.length) {
          files.file_and_group_ids.reduce((acc, file) => {
            acc[file.file.file_name] = file.file.id;
            return acc;
          }, fileMapResult);
        } else {
          done = true;
        }

        page += 1;
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

export const useFileContext = () => {
  return React.useContext(FileContext);
};
