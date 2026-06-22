import { notFound } from "next/navigation";
import { DocsShell } from "@/components/docs-shell";
import { getAllDocsPages, getDocsPage } from "@/lib/docs";

type DocsPageProps = {
  params: Promise<{
    slug: string;
  }>;
};

export async function generateStaticParams() {
  const pages = await getAllDocsPages();

  return pages.map((page) => ({
    slug: page.slug,
  }));
}

export async function generateMetadata({ params }: DocsPageProps) {
  const { slug } = await params;

  try {
    const page = await getDocsPage(slug);

    return {
      title: `${page.frontmatter.title} | STACC Documentation`,
      description: page.frontmatter.description,
    };
  } catch {
    return {
      title: "STACC Documentation",
    };
  }
}

export default async function DocsPage({ params }: DocsPageProps) {
  const { slug } = await params;
  const pagesPromise = getAllDocsPages();

  try {
    const pages = await pagesPromise;
    const page = pages.find((docsPage) => docsPage.slug === slug);

    if (!page) {
      notFound();
    }

    return (
      <div className="docs-route min-h-screen bg-white text-black px-4 py-4 sm:px-8 sm:py-8">
        <DocsShell page={page} pages={pages} />
      </div>
    );
  } catch {
    notFound();
  }
}
