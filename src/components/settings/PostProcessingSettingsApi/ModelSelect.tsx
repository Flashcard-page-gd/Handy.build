import React from "react";
import { components, type OptionProps } from "react-select";
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
  isTesting?: boolean;
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
    isTesting = false,
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

    const ModelOption = (props: OptionProps<ModelOption, false>) => (
      <components.Option {...props}>
        <div className="flex min-w-0 items-center gap-3">
          <span className="min-w-0 flex-1 truncate" title={props.data.label}>
            {props.children}
          </span>
          <button
            type="button"
            className="shrink-0 text-logo-primary hover:underline disabled:cursor-not-allowed disabled:opacity-50"
            disabled={isTesting}
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
            {isTesting ? testingLabel : testLabel}
          </button>
        </div>
      </components.Option>
    );

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
