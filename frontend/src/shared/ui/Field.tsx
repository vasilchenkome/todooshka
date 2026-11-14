import type { InputHTMLAttributes, ReactNode } from "react";

type FieldProps = InputHTMLAttributes<HTMLInputElement> & {
  label: ReactNode;
};

/** Form field: label above, underline below, accent on focus. */
export function Field({ label, ...input }: FieldProps) {
  return (
    <label className="block">
      <span className="mb-1 block text-[12px] text-steel">{label}</span>
      <input
        {...input}
        className="w-full border-b border-rule bg-transparent py-2 text-[15px] outline-none transition-colors focus:border-signal"
      />
    </label>
  );
}
