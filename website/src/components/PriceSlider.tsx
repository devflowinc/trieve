import { useEffect, useRef, useState } from "react";

// Format value for display (with commas)
export const formatValue = (val: number) => {
  // Format with Thousand, Million and Billion suffixes Remove the the extra decimal if it's a whole number
  if (val >= 1000000000) {
    let truncated = (val / 1000000000).toFixed(1);
    truncated = truncated.replace(".0", "");
    return truncated + "B";
  }
  if (val >= 1000000) {
    let truncated = (val / 1000000).toFixed(1);
    truncated = truncated.replace(".0", "");
    return truncated + "M";
  }
  if (val >= 1000) {
    let truncated = (val / 1000).toFixed(1);
    truncated = truncated.replace(".0", "");
    return truncated + "K";
  }
  return val.toLocaleString();
};

const PriceSlider = ({
  min = 10,
  max = 10000,
  markers = [10, 30, 100, 300, 1000, 3000, 10000],
  defaultValue = 1000,
  beforeValueText = "I have",
  afterValueText = "average support tickets per month",
  onChange,
  trackColorFilled = "bg-primary-700",
  trackColorEmpty = "bg-white",
  thumbColor = "bg-primary-900",
}) => {
  const [value, setValue] = useState(defaultValue);
  const [inputValue, setInputValue] = useState(defaultValue.toString());
  const [isDragging, setIsDragging] = useState(false);
  const [showTooltip, setShowTooltip] = useState(false);
  const sliderRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Parse value from formatted string
  const parseValue = (str: string) => {
    return parseInt(str.replace(/,/g, ""), 10);
  };

  const getPercentage = (val: number) => {
    if (val <= min) return 0;
    if (val >= max) return 100;
    const logMin = Math.log10(min);
    const logMax = Math.log10(max);
    const logVal = Math.log10(val);
    return ((logVal - logMin) / (logMax - logMin)) * 100;
  };

  const getValueFromPercentage = (percentage: number) => {
    const logMin = Math.log10(min);
    const logMax = Math.log10(max);
    const logVal = (percentage / 100) * (logMax - logMin) + logMin;
    return Math.round(Math.pow(10, logVal));
  };

  const generateMarkers = () => {
    // Use specific nice values that follow powers of 10
    const niceValues = markers.filter((v) => v >= min && v <= max);

    // Make sure we include min and max if they're not already in the list
    if (!niceValues.includes(min)) niceValues.unshift(min);
    if (!niceValues.includes(max)) niceValues.push(max);

    // Remove duplicates and sort
    return [...new Set(niceValues)].sort((a, b) => a - b);
  };

  const tickMarks = generateMarkers();

  const handleMouseDown: React.MouseEventHandler<HTMLDivElement> = (e) => {
    e.preventDefault();
    setIsDragging(true);
    // setShowTooltip(true);

    // Handle the initial click position
    if (sliderRef.current) {
      const rect = sliderRef.current.getBoundingClientRect();
      const percentage = Math.max(
        0,
        Math.min(100, ((e.clientX - rect.left) / rect.width) * 100),
      );
      const newValue = getValueFromPercentage(percentage);
      updateValue(newValue);
    }
  };

  const handleMouseUp: React.MouseEventHandler<HTMLDivElement> &
    React.TouchEventHandler<HTMLDivElement> = () => {
    setIsDragging(false);
    // setTimeout(() => setShowTooltip(false), 1000);
  };

  const handleMouseMove: React.MouseEventHandler<HTMLDivElement> = (e) => {
    if (!isDragging || !sliderRef.current) return;

    const rect = sliderRef.current.getBoundingClientRect();
    const percentage = Math.max(
      0,
      Math.min(100, ((e.clientX - rect.left) / rect.width) * 100),
    );
    const newValue = getValueFromPercentage(percentage);
    updateValue(newValue);
  };

  // Handle touch events
  const handleTouchStart: React.TouchEventHandler<HTMLDivElement> = (e) => {
    e.preventDefault();
    setIsDragging(true);
    setShowTooltip(true);

    if (sliderRef.current && e.touches[0]) {
      const rect = sliderRef.current.getBoundingClientRect();
      const percentage = Math.max(
        0,
        Math.min(100, ((e.touches[0].clientX - rect.left) / rect.width) * 100),
      );
      const newValue = getValueFromPercentage(percentage);
      updateValue(newValue);
    }
  };

  const handleTouchMove: React.TouchEventHandler<HTMLDivElement> = (e) => {
    if (!isDragging || !sliderRef.current || !e.touches[0]) return;

    const rect = sliderRef.current.getBoundingClientRect();
    const percentage = Math.max(
      0,
      Math.min(100, ((e.touches[0].clientX - rect.left) / rect.width) * 100),
    );
    const newValue = getValueFromPercentage(percentage);
    updateValue(newValue);
  };

  // Update both state and input values
  const updateValue = (newValue: number) => {
    // Ensure value is within bounds
    newValue = Math.max(min, Math.min(max, newValue));
    setValue(newValue);
    setInputValue(formatValue(newValue));
    if (onChange) onChange(newValue);
  };

  // Handle direct input changes
  const handleInputChange: React.ChangeEventHandler<HTMLInputElement> = (e) => {
    const raw = e.target.value;
    setInputValue(raw);

    // Try to parse the input value and update the slider in real-time
    const parsed = parseValue(raw);
    if (!isNaN(parsed) && parsed >= min && parsed <= max) {
      setValue(parsed);
      if (onChange) onChange(parsed);
    }
  };

  // When input loses focus, parse and validate the value
  const handleInputBlur = () => {
    const parsed = parseValue(inputValue);

    if (isNaN(parsed)) {
      // If invalid, revert to current value
      setInputValue(formatValue(value));
      return;
    }

    // Update with valid value
    updateValue(parsed);
  };

  // Handle Enter key to apply value
  const handleInputKeyDown: React.KeyboardEventHandler<HTMLInputElement> = (
    e,
  ) => {
    if (e.key === "Enter") {
      handleInputBlur();
      (e.target as HTMLInputElement).blur();
    } else if (e.key === "Escape") {
      // Revert on Escape
      setInputValue(formatValue(value));
      (e.target as HTMLInputElement).blur();
    }
  };

  useEffect(() => {
    if (isDragging) {
      document.body.classList.add("select-none");

      const handleMouseMoveGlobal = (e: MouseEvent) => {
        e.preventDefault();

        if (!isDragging || !sliderRef.current) return;

        const rect = sliderRef.current.getBoundingClientRect();
        const percentage = Math.max(
          0,
          Math.min(100, ((e.clientX - rect.left) / rect.width) * 100),
        );
        const newValue = getValueFromPercentage(percentage);
        updateValue(newValue);
      };

      const handleTouchMoveGlobal = (e: TouchEvent) => {
        if (!isDragging || !sliderRef.current || !e.touches[0]) return;

        const rect = sliderRef.current.getBoundingClientRect();
        const percentage = Math.max(
          0,
          Math.min(
            100,
            ((e.touches[0].clientX - rect.left) / rect.width) * 100,
          ),
        );
        const newValue = getValueFromPercentage(percentage);
        updateValue(newValue);
      };

      const handleMouseUpGlobal = () => {
        document.body.classList.remove("select-none");
        setIsDragging(false);
        setTimeout(() => setShowTooltip(false), 1000);
      };

      const handleTouchEndGlobal = () => {
        document.body.classList.remove("select-none");
        setIsDragging(false);
        setTimeout(() => setShowTooltip(false), 1000);
      };

      window.addEventListener("mousemove", handleMouseMoveGlobal);
      window.addEventListener("mouseup", handleMouseUpGlobal);
      window.addEventListener("touchmove", handleTouchMoveGlobal, {
        passive: false,
      });
      window.addEventListener("touchend", handleTouchEndGlobal);

      return () => {
        window.removeEventListener("mousemove", handleMouseMoveGlobal);
        window.removeEventListener("mouseup", handleMouseUpGlobal);
        window.removeEventListener("touchmove", handleTouchMoveGlobal);
        window.removeEventListener("touchend", handleTouchEndGlobal);
      };
    }
  }, [isDragging, onChange]);

  // Initialize input value when default value changes
  useEffect(() => {
    setInputValue(formatValue(defaultValue));
  }, [defaultValue]);

  const thumbPercentage = getPercentage(value);

  return (
    <div className="w-full mx-auto px-6 py-8 md:px-8 md:py-10 rounded-lg bg-primary-300">
      <div className="mb-6 ">
        <div className="flex flex-wrap md:flex-nowrap gap-x-2 items-center mb-4">
          <div className="text-lg font-medium">{beforeValueText}</div>
          <div>
            <input
              ref={inputRef}
              type="text"
              className="text-lg font-bold bg-primary-100 text-primary-900 px-2 py-0.5 w-24 rounded-full text-center"
              value={inputValue}
              onChange={handleInputChange}
              onBlur={handleInputBlur}
              onKeyDown={handleInputKeyDown}
              aria-label="Set value"
            />
          </div>
          <div className="text-lg font-medium">{afterValueText}</div>
        </div>

        <div className="relative mt-1">
          {/* Track background (lighter color) */}
          <div
            className={`absolute inset-0 rounded-full ${trackColorEmpty}`}
          ></div>

          {/* Track filled part (darker color) up to the thumb */}
          <div
            className={`absolute inset-y-0 left-0 rounded-full ${trackColorFilled}`}
            style={{ width: `${thumbPercentage}%` }}
          ></div>

          {/* Track container for events */}
          <div
            ref={sliderRef}
            className="relative h-4 rounded-full cursor-pointer select-none z-10"
            style={{ backgroundColor: "transparent" }}
            onMouseDown={handleMouseDown}
            onMouseUp={handleMouseUp}
            onMouseMove={handleMouseMove}
            onTouchStart={handleTouchStart}
            onTouchEnd={handleMouseUp}
            onTouchMove={handleTouchMove}
          >
            {/* Thumb */}
            <div
              className={`absolute -top-1 h-6 w-6 bg-primary-900 border-4 border-primary-300 rounded-full cursor-grab transform -translate-x-1/2 z-20`}
              style={{ left: `${thumbPercentage}%` }}
            ></div>

            {/* Tooltip */}
            {showTooltip && (
              <div
                className="absolute -top-8 px-2 py-1 bg-primary-900  text-xs rounded transform -translate-x-1/2 z-30"
                style={{ left: `${thumbPercentage}%` }}
              >
                {formatValue(value)}
              </div>
            )}
          </div>

          {/* Track markers */}
          <div className="relative inset-x-0">
            {tickMarks.map((marker, index) => (
              <div
                key={marker}
                className="flex flex-col items-center mt-2 "
                style={{
                  position: "absolute",
                  left: `${getPercentage(marker)}%`,
                  transform: "translateX(-50%)",
                }}
              >
                <div className="flex-auto h-3 w-0.5 bg-black mx-auto"></div>
                <div className="flex-auto text-sm">{formatValue(marker)}</div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default PriceSlider;
