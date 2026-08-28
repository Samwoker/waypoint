import { DocCategory, DocHeading, DocItem, SearchResultItem } from './types';

// Eagerly import all Markdown files from the docs directory
const rawDocs = import.meta.glob('../../../docs/**/*.md', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

// Helper to extract clean slug ID from heading text
export function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w\s-]/g, '')
    .trim()
    .replace(/\s+/g, '-');
}

// Extract headings from raw markdown string
function extractHeadings(markdown: string): DocHeading[] {
  const headings: DocHeading[] = [];
  const lines = markdown.split('\n');

  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith('## ')) {
      const text = trimmed.replace(/^##\s+/, '').replace(/[#*`_]/g, '').trim();
      if (text) {
        headings.push({
          id: slugify(text),
          text,
          level: 2,
        });
      }
    } else if (trimmed.startsWith('### ')) {
      const text = trimmed.replace(/^###\s+/, '').replace(/[#*`_]/g, '').trim();
      if (text) {
        headings.push({
          id: slugify(text),
          text,
          level: 3,
        });
      }
    }
  }

  return headings;
}

// Extract title and description
function extractMetadata(markdown: string, fallbackName: string): { title: string; description?: string } {
  const lines = markdown.split('\n');
  let title = fallbackName;
  let description: string | undefined = undefined;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    if (line.startsWith('# ') && title === fallbackName) {
      title = line.replace(/^#\s+/, '').replace(/[#*`_]/g, '').trim();
    } else if (line && !line.startsWith('#') && !line.startsWith('>') && !line.startsWith('```') && !description) {
      description = line.slice(0, 160);
    }
    if (title !== fallbackName && description) break;
  }

  return { title, description };
}

// Category ordering definition
const CATEGORY_ORDER: { [key: string]: { label: string; order: number } } = {
  introduction: { label: 'INTRODUCTION', order: 1 },
  'getting-started': { label: 'GETTING STARTED', order: 2 },
  concepts: { label: 'CORE CONCEPTS', order: 3 },
  api: { label: 'API REFERENCE', order: 4 },
  integrations: { label: 'SDKs & INTEGRATIONS', order: 5 },
  security: { label: 'SECURITY & HARDENING', order: 6 },
  operations: { label: 'OPERATIONS & DEPLOYMENT', order: 7 },
  troubleshooting: { label: 'TROUBLESHOOTING', order: 8 },
  reference: { label: 'REFERENCE & FAQ', order: 9 },
};

// Process all markdown files
const parsedDocItems: DocItem[] = [];

for (const [filePath, content] of Object.entries(rawDocs)) {
  // Normalize path: e.g. "../../../docs/getting-started/quickstart.md" -> "getting-started/quickstart"
  const cleanPath = filePath
    .replace(/^(\.\.\/)+docs\//, '')
    .replace(/\.md$/, '');

  if (cleanPath === 'README') {
    // Root README will be accessible via /docs or /docs/README
    const { title, description } = extractMetadata(content, 'Documentation Overview');
    parsedDocItems.push({
      id: 'readme',
      slug: 'readme',
      route: '/docs/readme',
      title: 'Documentation Overview',
      description,
      rawContent: content,
      headings: extractHeadings(content),
      category: 'introduction',
      order: 0,
    });
    continue;
  }

  const parts = cleanPath.split('/');
  const folder = parts[0] || 'general';
  const fileId = parts[1] || parts[0];

  const fallbackTitle = fileId
    .split('-')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');

  const { title, description } = extractMetadata(content, fallbackTitle);

  parsedDocItems.push({
    id: fileId,
    slug: cleanPath,
    route: `/docs/${cleanPath}`,
    title,
    description,
    rawContent: content,
    headings: extractHeadings(content),
    category: folder,
  });
}

// Group into categories
const categoryMap = new Map<string, DocItem[]>();

for (const item of parsedDocItems) {
  const cat = item.category;
  if (!categoryMap.has(cat)) {
    categoryMap.set(cat, []);
  }
  categoryMap.get(cat)!.push(item);
}

export const docCategories: DocCategory[] = Array.from(categoryMap.entries())
  .map(([catKey, items]) => {
    const meta = CATEGORY_ORDER[catKey] || { label: catKey.toUpperCase(), order: 99 };
    // Sort items within category logically
    items.sort((a, b) => {
      // overview / quickstart first
      if (a.id === 'overview' || a.id === 'installation') return -1;
      if (b.id === 'overview' || b.id === 'installation') return 1;
      if (a.id === 'quickstart') return -1;
      if (b.id === 'quickstart') return 1;
      return a.title.localeCompare(b.title);
    });

    return {
      id: catKey,
      label: meta.label,
      order: meta.order,
      items,
    };
  })
  .sort((a: any, b: any) => a.order - b.order);

export const allDocs: DocItem[] = parsedDocItems;

// Helper to find document by route or slug
export function getDocBySlug(slug: string): DocItem | undefined {
  const normalized = slug.replace(/^\/docs\//, '').replace(/^\//, '').replace(/\.md$/, '');
  if (!normalized || normalized === 'readme') {
    return allDocs.find((d) => d.slug === 'readme' || d.slug === 'introduction/overview');
  }
  return (
    allDocs.find((d) => d.slug === normalized) ||
    allDocs.find((d) => d.id === normalized) ||
    allDocs.find((d) => d.route === `/docs/${normalized}`)
  );
}

// Helper to get previous and next document in sequential order
export function getPrevNextDocs(currentSlug: string): { prev?: DocItem; next?: DocItem } {
  // Flatten docs in category order
  const flatDocs: DocItem[] = [];
  for (const cat of docCategories) {
    for (const item of cat.items) {
      if (item.slug !== 'readme') {
        flatDocs.push(item);
      }
    }
  }

  const currentIndex = flatDocs.findIndex(
    (d) => d.slug === currentSlug || d.id === currentSlug || d.route === `/docs/${currentSlug}`
  );

  if (currentIndex === -1) return {};

  return {
    prev: currentIndex > 0 ? flatDocs[currentIndex - 1] : undefined,
    next: currentIndex < flatDocs.length - 1 ? flatDocs[currentIndex + 1] : undefined,
  };
}

// Search across all documentation
export function searchDocs(query: string): SearchResultItem[] {
  if (!query || query.trim().length === 0) return [];
  const q = query.toLowerCase().trim();
  const results: SearchResultItem[] = [];

  for (const doc of allDocs) {
    let score = 0;
    let headingMatch: string | undefined = undefined;
    let snippet = doc.description || '';

    // Title match
    if (doc.title.toLowerCase().includes(q)) {
      score += 50;
    }

    // Slug / Route match
    if (doc.slug.toLowerCase().includes(q)) {
      score += 30;
    }

    // Heading match
    for (const h of doc.headings) {
      if (h.text.toLowerCase().includes(q)) {
        score += 20;
        if (!headingMatch) {
          headingMatch = h.text;
        }
      }
    }

    // Content match
    const lowerContent = doc.rawContent.toLowerCase();
    const matchIndex = lowerContent.indexOf(q);
    if (matchIndex !== -1) {
      score += 10;
      const start = Math.max(0, matchIndex - 40);
      const end = Math.min(doc.rawContent.length, matchIndex + q.length + 60);
      snippet = '...' + doc.rawContent.slice(start, end).replace(/[#*`_]/g, '') + '...';
    }

    if (score > 0) {
      results.push({
        item: doc,
        headingMatch,
        snippet,
        score,
      });
    }
  }

  return results.sort((a, b) => b.score - a.score).slice(0, 8);
}
