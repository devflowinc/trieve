import React, { useEffect, useState } from "react";
import { PdfChunk } from "../../utils/types";
import { useFileContext } from "../../utils/hooks/file-context";
import { FileDTO } from "trieve-ts-sdk";
import { useModalState } from "../../utils/hooks/modal-context";
import { cached } from "../../utils/cache";
import { PdfSpotlight } from "react-pdf-spotlight";
import { file } from "bun";

type Props = {
  item: PdfChunk;
  requestID: string;
  index: number;
  className?: string;
};

function extractMarkedContent(text: string): string {
  const regex = /<mark><b>(.*?)<\/b><\/mark>/;
  const match = text.match(regex);
  return match ? match[1] : "";
}

const getPresignedUrl = async (
  baseUrl: string,
  datasetId: string,
  fileId: string,
  apiKey: string,
) => {
  const params = {
    content_type: "application/pdf",
  };
  const queryParams = new URLSearchParams(params).toString();
  const result = await fetch(`${baseUrl}/api/file/${fileId}?${queryParams}`, {
    headers: {
      "TR-Dataset": datasetId,
      Authorization: `Bearer ${apiKey}`,
    },
  });

  if (!result.ok) {
    throw new Error("Error fetching presigned url");
  }

  const presignedUrl = (await result.json()) as FileDTO;

  return presignedUrl.s3_url;
};

export const PdfItem = (props: Props) => {
  const [presigned, setPresigned] = useState<string | null>(null);
  const toHighlight = extractMarkedContent(props.item.chunk.highlight || "");
  const fileCtx = useFileContext();
  const state = useModalState();

  useEffect(() => {
    const getPresigned = async () => {
      const presignedUrlResult = await cached(() => {
        return getPresignedUrl(
          state.props.baseUrl || "http://localhost:8090",
          state.props.datasetId,
          fileCtx.files[props.item.chunk.metadata.file_name],
          state.props.apiKey,
        );
      }, `file-presigned:${props.item.chunk.metadata.file_name}`);
      setPresigned(presignedUrlResult);
    };

    getPresigned();
  }, []);

  return (
    <div>
      {presigned && (
        <div className="max-w-[400px]">
          <PdfSpotlight
            padding={{
              horizontal: 100,
            }}
            page={props.item.chunk.metadata.page}
            searchFor={toHighlight}
            url={presigned}
          ></PdfSpotlight>
        </div>
      )}
    </div>
  );
};
