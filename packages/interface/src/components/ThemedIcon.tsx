import type { ImgHTMLAttributes } from "react";
import { useThemedIcon } from "../hooks/useTheme";

/** An icon image that switches to its `_Light` asset under light themes. */
export function ThemedIcon({ src, ...props }: ImgHTMLAttributes<HTMLImageElement> & { src: string }) {
  const themed = useThemedIcon();
  return <img {...props} src={themed(src)} />;
}
