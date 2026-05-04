/**
 * Single-field auto-save wrapper.
 */

import { useRef, useState, type ReactNode } from 'react';

import { SettingRow } from './index';
import { useDebouncedSave } from '../hooks/useDebouncedSave';
import type { RawAppConfig } from '../types';

type Primitive = string | number | boolean | string[];

interface SaveFieldProps<TValue extends Primitive> {
  section: string;
  fieldKey: string;
  label: string;
  helper?: string;
  vertical?: boolean;
  rightAlign?: boolean;
  initialValue: TValue;
  resyncToken: number;
  onSaved: (next: RawAppConfig) => void;
  render: (
    value: TValue,
    setValue: (next: TValue) => void,
    errored: boolean,
  ) => ReactNode;
}

export function SaveField<TValue extends Primitive>({
  section,
  fieldKey,
  label,
  helper,
  vertical,
  rightAlign,
  initialValue,
  resyncToken,
  onSaved,
  render,
}: SaveFieldProps<TValue>) {
  const [value, setValue] = useState<TValue>(initialValue);

  const { error, resetTo } = useDebouncedSave(section, fieldKey, value, {
    onSaved,
  });

  const lastTokenRef = useRef(resyncToken);
  if (lastTokenRef.current !== resyncToken) {
    lastTokenRef.current = resyncToken;
    setValue(initialValue);
    resetTo(initialValue);
  }

  return (
    <SettingRow
      label={label}
      helper={helper}
      vertical={vertical}
      rightAlign={rightAlign}
      error={error}
    >
      {render(value, setValue, error !== null)}
    </SettingRow>
  );
}