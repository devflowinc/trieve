import React, { useState } from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { cn } from "../../utils/styles";

export const UploadAudio = () => {
  const { mode, setAudioBase64 } = useModalState();

  const [isRecording, setIsRecording] = useState(false);
  const [mediaRecorder, setMediaRecorder] = useState<MediaRecorder | null>(
    null,
  );

  const startRecording = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: true,
      });

      const recorder = new MediaRecorder(stream);
      const audioChunks: BlobPart[] = [];

      recorder.ondataavailable = (event) => {
        audioChunks.push(event.data);
      };

      recorder.onstop = () => {
        stream.getTracks().forEach((track) => track.stop());
        setAudioBase64("");
        const audioBlob = new Blob(audioChunks, { type: "audio/mp3" });
        const reader = new FileReader();
        reader.readAsDataURL(audioBlob);
        reader.onloadend = () => {
          let base64data = reader.result as string;
          base64data = base64data?.split(",")[1];
          setAudioBase64(base64data);
        };
      };

      setMediaRecorder(recorder);
      recorder.start();
      setIsRecording(true);
    } catch (err) {
      console.error("Error accessing microphone:", err);
      alert(
        "Error accessing microphone. Please make sure you have granted microphone permissions.",
      );
    }
  };

  const stopRecording = () => {
    if (mediaRecorder && mediaRecorder?.state !== "inactive") {
      mediaRecorder?.stop();
      setIsRecording(false);
    }
  };

  const toggleRecording = () => {
    if (isRecording) {
      stopRecording();
    } else {
      void startRecording();
    }
  };

  return (
    <div>
      <button
        type="button"
        className={cn(`tv-rounded${
          mode === "chat" ? " tv-right-16 tv-top-[0.825rem] tv-absolute" : ""
        } tv-z-20 cursor-pointer ${
          isRecording
            ? "tv-text-red-500"
            : "tv-dark-text-white tv-text-zinc-700"
        }
        `)}
        onClick={(e) => {
          e.preventDefault();
          toggleRecording();
        }}>
        <i className="fa-solid fa-microphone"> </i>
      </button>
    </div>
  );
};
