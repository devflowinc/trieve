import Papa from "papaparse";

type ChunkData = {
  "Chunk HTML": string;
  Link: string;
  "Tag Set": string;
  "Tracking ID": string;
  Metadata: string; // Assuming Metadata can be a string or parsed as an object
};

type Data = {
  chunk_html: string;
  link: string;
  tag_set: string[];
  tracking_id: string;
  metadata: unknown;
  upsert_by_tracking_id: boolean;
};

type UploadDataArray = Data[][];

const api_host = import.meta.env.VITE_API_HOST as unknown as string;

const sendDataToTrieve = async (
  data: UploadDataArray,
  datasetId: string,
  reportProgress?: (progress: number) => void,
) => {
  for (let i = 0; i < data.length; i++) {
    await fetch(`${api_host}/chunk`, {
      method: "POST",
      headers: {
        "TR-Dataset": datasetId,
        "Content-Type": "application/json",
      },
      credentials: "include",
      body: JSON.stringify(data[i]),
    });
    if (reportProgress) {
      reportProgress(((i + 1) / data.length) * 100);
    }
  }
};

const transformRow = (row: { data: ChunkData }): Data | undefined => {
  if (!row.data) return undefined;
  if (row.data["Chunk HTML"] === "") return undefined;
  try {
    return {
      chunk_html: row.data["Chunk HTML"].replace(/;/g, ","),
      link: row.data.Link.replace(/;/g, ","),
      tag_set: row.data["Tag Set"].split("|"),
      tracking_id: row.data["Tracking ID"],
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      metadata: JSON.parse(row.data.Metadata.replace(/;/g, ",")),
      upsert_by_tracking_id: true,
    };
  } catch (_e) {
    return undefined;
  }
};

export const uploadSampleData = ({
  datasetId,
  reportProgress,
}: {
  datasetId: string;
  reportProgress?: (progress: number) => void;
}): Promise<boolean> => {
  return new Promise<boolean>((resolve, reject) => {
    const dataToUpload: UploadDataArray = [[]];
    let arrayIdx = 0;
    Papa.parse(
      "https://gist.githubusercontent.com/densumesh/16f667de9149902f989250a8a1c50969/raw/4631cf5894a9fd473b708a4b372cd58f84808591/shortened_yc_example.csv",
      {
        download: true,
        header: true,
        step: (row: { data: ChunkData }) => {
          const transformedRow = transformRow(row);
          if (dataToUpload[arrayIdx].length >= 120) {
            dataToUpload.push([]);
            arrayIdx++;
          }
          if (transformedRow) {
            dataToUpload[arrayIdx].push(transformedRow);
          }
        },
        complete: function () {
          void sendDataToTrieve(dataToUpload, datasetId, reportProgress).then(
            () => {
              resolve(true);
            },
          );
        },
        error: () => {
          reject(false);
        },
      },
    );
  });
};
