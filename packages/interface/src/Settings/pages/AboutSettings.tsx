import { motion } from "framer-motion";
import type { MouseEvent } from "react";
import { WingDriveLogo } from "@wingdrive/assets/images";
import { CircleButton } from "@wingdrive/primitives";
import { ChatsCircle, GithubLogo } from "@phosphor-icons/react";
import contributors from "../../contributors.json";
import { usePlatform } from "../../contexts/PlatformContext";
import { useTranslation } from "../../i18n";

export function AboutSettings() {
  const { t } = useTranslation();
  const platform = usePlatform();
  // The Tauri webview ignores target="_blank"; route links to the system browser.
  const openExternal = (event: MouseEvent<HTMLAnchorElement>) => {
    event.preventDefault();
    platform.openLink(event.currentTarget.href);
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-[600px]">

      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.6, ease: "easeOut" }}
        className="relative w-64 h-64 mb-8"
      >

        <div className="absolute inset-[8%] z-0">
          <img
            src={WingDriveLogo}
            alt="WingDrive"
            className="w-full h-full object-contain select-none"
            draggable={false}
          />
        </div>

      </motion.div>

      {/* Branding */}
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.3 }}
        className="text-center mb-6"
      >
        <h3 className="text-2xl font-bold text-ink mb-2">WingDrive</h3>
        <p className="text-sm text-ink-dull">
          {t("about.tagline")}
        </p>
      </motion.div>

      {/* Manifesto */}
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.35 }}
        className="max-w-md text-center mb-8 px-4"
      >
        <p className="text-sm text-ink-dull leading-relaxed">
          {t("about.manifesto")}
        </p>
      </motion.div>

      {/* Links */}
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.4 }}
        className="flex gap-3 mb-6"
      >
        <a
          href="https://github.com/bernardopg/wingdrive"
          target="_blank"
          rel="noopener noreferrer"
          onClick={openExternal}
        >
          <CircleButton icon={GithubLogo}>
            GitHub
          </CircleButton>
        </a>
        <a
          href="https://github.com/bernardopg/wingdrive/discussions"
          target="_blank"
          rel="noopener noreferrer"
          onClick={openExternal}
        >
          <CircleButton icon={ChatsCircle}>
            {t("about.discussions")}
          </CircleButton>
        </a>
      </motion.div>

      {/* Contributors */}
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.45 }}
        className="max-w-lg text-center mb-8 px-4"
      >
        <p className="text-[11px] leading-relaxed text-ink-faint">
          {contributors.map((c: { name: string; github: string }, i) => (
            <span key={c.github}>
              {i > 0 && " · "}
              <a
                href={`https://github.com/${c.github}`}
                target="_blank"
                rel="noopener noreferrer"
                onClick={openExternal}
                title={`@${c.github}`}
                className="hover:text-ink-dull transition-colors"
              >
                {c.name}
              </a>
            </span>
          ))}
        </p>
      </motion.div>

      {/* License */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.5, delay: 0.55 }}
        className="text-center"
      >
        <a
          href="https://github.com/bernardopg/wingdrive/blob/main/LICENSE"
          target="_blank"
          rel="noopener noreferrer"
          onClick={openExternal}
          className="text-sm text-ink-faint hover:text-ink-dull transition-colors"
        >
          Apache-2.0
        </a>
      </motion.div>
    </div>
  );
}
