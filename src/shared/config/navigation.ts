export const PAGES = ["board", "discarded", "profile", "resumes", "prep", "settings"] as const;
export type PageId = (typeof PAGES)[number];
