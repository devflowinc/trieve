interface ProgressBarProps {
  max: number;
  progress: number;
  width?: string;
}

export const ProgressBar = (props: ProgressBarProps) => {
  const percentage = () => {
    if (props.max <= 0) return 0;
    return Math.min(100, Math.max(0, (props.progress / props.max) * 100));
  };

  return (
    <div
      class="h-2.5 rounded-full bg-gray-200 dark:bg-gray-700"
      style={{ width: props.width || "100%" }}
    >
      <div
        class="h-2.5 rounded-full bg-magenta transition-all duration-300 ease-in-out"
        style={{ width: `${percentage()}%` }}
      />
    </div>
  );
};
