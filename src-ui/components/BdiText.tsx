import React from "react";

export interface BdiTextProps {
  text: string;
  className?: string;
  testId?: string;
}

/**
 * Isolates bidirectional text (filenames, paths, URLs combining Arabic, English, and numbers)
 * using `<bdi dir="auto">` so surrounding RTL/LTR direction does not scramble punctuation or extensions.
 */
export function BdiText({
  text,
  className,
  testId,
}: BdiTextProps): React.ReactElement {
  return (
    <bdi dir="auto" className={className} data-testid={testId}>
      {text}
    </bdi>
  );
}
