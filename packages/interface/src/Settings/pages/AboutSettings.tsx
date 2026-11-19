import type { MouseEvent } from "react";
import { WingDriveLogo } from "@wingdrive/assets/images";
import { ChatsCircle, GithubLogo, Globe } from "@phosphor-icons/react";
import contributors from "../../contributors.json";
import { usePlatform } from "../../contexts/PlatformContext";
import { useTranslation } from "../../i18n";
import { SettingsHeader, SettingsSection, buttonClass } from "../components";

export function AboutSettings() {
  const { t } = useTranslation();
  const platform = usePlatform();
  const openExternal = (event: MouseEvent<HTMLAnchorElement>) => {
    event.preventDefault();
    platform.openLink(event.currentTarget.href);
  };
  const links = [
    { label: t("about.website"), icon: Globe, href: "https://wingdrive.org" },
    { label: "GitHub", icon: GithubLogo, href: "https://github.com/bernardopg/wingdrive" },
    { label: t("about.discussions"), icon: ChatsCircle, href: "https://github.com/bernardopg/wingdrive/discussions" },
  ];

  return (
    <div className="space-y-6">
      <SettingsHeader title={t("sections.about")} description={t("about.description")} />
      <SettingsSection>
        <div className="flex flex-wrap items-center gap-4 p-4">
          <img src={WingDriveLogo} alt="" className="size-16 shrink-0 object-contain" draggable={false} />
          <div className="min-w-0 flex-1 basis-48">
            <h3 className="text-lg font-semibold text-ink">WingDrive</h3>
            <p className="mt-1 text-sm text-ink-dull">{t("about.overview")}</p>
          </div>
        </div>
        <div className="flex flex-wrap gap-2 p-4">
          {links.map(({ label, icon: Icon, href }) => (
            <a key={href} href={href} onClick={openExternal} className={`${buttonClass} inline-flex items-center gap-2 bg-app-button text-ink hover:bg-app-hover`}>
              <Icon className="size-4" aria-hidden />
              {label}
            </a>
          ))}
        </div>
      </SettingsSection>
      <SettingsSection title={t("about.credits")}>
        <ul className="flex flex-wrap gap-x-4 gap-y-2 p-4 text-sm text-ink-dull">
          {contributors.map((contributor) => (
            <li key={contributor.github}>
              <a href={`https://github.com/${contributor.github}`} onClick={openExternal} className="underline-offset-2 hover:underline">
                {contributor.name}
              </a>
            </li>
          ))}
        </ul>
        <p className="p-4 text-sm text-ink-dull">
          {t("about.license")}{" "}
          <a href="https://github.com/bernardopg/wingdrive/blob/main/LICENSE" onClick={openExternal} className="text-accent underline">
            Apache-2.0
          </a>
        </p>
      </SettingsSection>
    </div>
  );
}
