import React, { useState } from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { cn } from "../../utils/styles";

export const UploadAudio = () => {
  const { mode, setAudioBase64 } = useModalState();

  const [isRecording, setIsRecording] = useState(false);
  const [mediaRecorder, setMediaRecorder] = useState<MediaRecorder | null>(
    null
  );

  const startRecording = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: true,
      });

      const recorder = new MediaRecorder(stream, {
        mimeType: "audio/mp4",
      });
      const audioChunks: BlobPart[] = [];

      recorder.ondataavailable = (event) => {
        audioChunks.push(event.data);
      };

      recorder.onstop = () => {
        stream.getTracks().forEach((track) => track.stop());
        setAudioBase64("");
        const audioBlob = new Blob(audioChunks, { type: "audio/mp4" });
        const reader = new FileReader();
        reader.readAsDataURL(audioBlob);
        reader.onloadend = () => {
          let base64data = reader.result as string;
          base64data = base64data?.split(",")[1];
          setAudioBase64(base64data);
        };
      };

      setMediaRecorder(recorder);
      recorder.start(100);
      setIsRecording(true);
    } catch (err) {
      console.error("Error accessing microphone:", err);
      alert(
        "Error accessing microphone. Please make sure you have granted microphone permissions."
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
    <div
      aria-role="button"
      className={cn(
        "tv-rounded tv-z-20 tv-cursor-pointer",
        mode === "chat" && "tv-right-16 tv-top-[0.825rem] tv-absolute",
        isRecording ? "tv-text-red-500" : "tv-dark-text-white tv-text-zinc-700"
      )}
      onClick={(e) => {
        e.preventDefault();
        toggleRecording();
      }}
    >
      <div
        onClick={(e) => {
          e.preventDefault();
          toggleRecording();
        }}
        className="tv-block md:tv-hidden tv-top-0 tv-bottom-0 tv-left-0 tv-right-0 tv-scale-125 tv-pr-3 tv-pl-6 tv-py-8 -tv-translate-x-5 tv-absolute"
      ></div>
      <i className="fa-solid fa-microphone"> </i>
    </div>
  );
};
