import React from "react";
import { components, type OptionProps } from "react-select";
import { Check } from "lucide-react";
import type { ModelOption } from "./types";
import { Select } from "../../ui/Select";

type ModelSelectProps = {
  value: string;
  options: ModelOption[];
  disabled?: boolean;
  placeholder?: string;
  isLoading?: boolean;
  onSelect: (value: string) => void;
  onCreate: (value: string) => void;
  onBlur: () => void;
  onTest: (value: string) => void;
  testingModel?: string | null;
  successModel?: string | null;
  testLabel: string;
  testingLabel: string;
  className?: string;
};

export const ModelSelect: React.FC<ModelSelectProps> = React.memo(
  ({
    value,
    options,
    disabled,
    placeholder,
    isLoading,
    onSelect,
    onCreate,
    onBlur,
    onTest,
    testingModel = null,
    successModel = null,
    testLabel,
    testingLabel,
    className = "flex-1 min-w-[360px]",
  }) => {
    const handleCreate = (inputValue: string) => {
      const trimmed = inputValue.trim();
      if (!trimmed) return;
      onCreate(trimmed);
    };

    const computedClassName = `text-sm ${className}`;

    const ModelOption = (props: OptionProps<ModelOption, false>) => {
      const isThisTesting = testingModel === props.data.value;
      const isThisSuccess = successModel === props.data.value;
      const isAnyTesting = Boolean(testingModel);

      return (
        <components.Option {...props}>
          <div className="flex min-w-0 items-center justify-between gap-3 w-full">
            <span className="min-w-0 flex-1 truncate" title={props.data.label}>
              {props.children}
            </span>
            {isThisSuccess ? (
              <span className="inline-flex shrink-0 items-center text-green-500">
                <Check className="h-4 w-4" />
              </span>
            ) : (
              <button
                type="button"
                className="shrink-0 text-xs font-medium lowercase text-logo-primary hover:text-logo-primary/80 disabled:cursor-not-allowed disabled:opacity-50 transition-colors"
                disabled={isAnyTesting}
                onMouseDown={(event) => {
                  event.preventDefault();
                  event.stopPropagation();
                }}
                onClick={(event) => {
                  event.preventDefault();
                  event.stopPropagation();
                  onTest(props.data.value);
                }}
              >
                {isThisTesting ? testingLabel : testLabel}
              </button>
            )}
          </div>
        </components.Option>
      );
    };

    return (
      <Select
        className={computedClassName}
        value={value || null}
        options={options}
        onChange={(selected) => onSelect(selected ?? "")}
        onCreateOption={handleCreate}
        onBlur={onBlur}
        placeholder={placeholder}
        disabled={disabled}
        isLoading={isLoading}
        isCreatable
        components={{ Option: ModelOption }}
        formatCreateLabel={(input) => `Use "${input}"`}
      />
    );
  },
);

ModelSelect.displayName = "ModelSelect";
