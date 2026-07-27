import type { Metadata, Viewport } from "next";
import { Anton, IBM_Plex_Mono } from "next/font/google";
import { CommandSearchProvider } from "@/components/command-search";
import { getSearchItems } from "@/lib/search";
import "./globals.css";

export const metadata: Metadata = {
  title: "STACC Documentation",
  description: "STACC agent configuration suite and documentation.",
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  colorScheme: "light dark",
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#ffffff" },
    { media: "(prefers-color-scheme: dark)", color: "#080808" },
  ],
};

const anton = Anton({
  subsets: ["latin"],
  weight: "400",
  variable: "--font-anton",
});

const ibmPlexMono = IBM_Plex_Mono({
  subsets: ["latin"],
  weight: ["500", "700"],
  variable: "--font-ibm-plex-mono",
});

const themeColors = {
  dark: {
    background: "#080808",
    text: "#f7f7f7",
  },
  light: {
    background: "#ffffff",
    text: "#000000",
  },
} as const;

const themeScript = `
(function () {
  var colors = {
    dark: { background: "${themeColors.dark.background}", text: "${themeColors.dark.text}" },
    light: { background: "${themeColors.light.background}", text: "${themeColors.light.text}" }
  };

  function applyResolvedTheme(choice, resolved) {
    document.documentElement.dataset.theme = resolved;
    document.documentElement.dataset.themeChoice = choice;
    document.documentElement.style.colorScheme = resolved;
    document.documentElement.style.backgroundColor = colors[resolved].background;
    document.documentElement.style.color = colors[resolved].text;
  }

  try {
    var stored = window.localStorage.getItem("stacc-theme");
    var choice = stored === "light" || stored === "dark" || stored === "system" ? stored : "system";
    var systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    var resolved = choice === "system" ? (systemDark ? "dark" : "light") : choice;
    applyResolvedTheme(choice, resolved);
  } catch (error) {
    if (!(error instanceof Error)) {
      throw error;
    }

    applyResolvedTheme("system", "light");
  }
})();
`;

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const searchItems = await getSearchItems();

  return (
    <html lang="en" className={`${anton.variable} ${ibmPlexMono.variable}`} suppressHydrationWarning>
      <head>
        <script id="stacc-theme-script" dangerouslySetInnerHTML={{ __html: themeScript }} />
      </head>
      <body>
        <CommandSearchProvider items={searchItems}>{children}</CommandSearchProvider>
      </body>
    </html>
  );
}
