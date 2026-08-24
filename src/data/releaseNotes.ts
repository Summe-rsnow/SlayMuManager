export interface ReleaseNoteConfig {
  version: string
  date: string
  tagKey?: string
  titleKey: string
  highlightsKeys?: string[]
  changes: {
    type: "feat" | "perf" | "ui" | "fix"
    textKey: string
  }[]
}

export const releaseNotesConfig: ReleaseNoteConfig[] = [
  {
    version: "1.8.5",
    date: "2026-08-24",
    titleKey: "releaseNotes.v1_8_5.title",
    highlightsKeys: [
      "releaseNotes.v1_8_5.highlights.0",
      "releaseNotes.v1_8_5.highlights.1",
    ],
    changes: [
      { type: "feat", textKey: "releaseNotes.v1_8_5.changes.0" },
      { type: "feat", textKey: "releaseNotes.v1_8_5.changes.1" },
      { type: "perf", textKey: "releaseNotes.v1_8_5.changes.2" },
      { type: "fix", textKey: "releaseNotes.v1_8_5.changes.3" },
    ],
  },
  {
    version: "1.8.4",
    date: "2026-08-24",
    titleKey: "releaseNotes.v1_8_4.title",
    changes: [
      { type: "feat", textKey: "releaseNotes.v1_8_4.changes.0" },
      { type: "fix", textKey: "releaseNotes.v1_8_4.changes.1" },
      { type: "perf", textKey: "releaseNotes.v1_8_4.changes.2" },
    ],
  },
  {
    version: "1.8.3",
    date: "2026-08-20",
    titleKey: "releaseNotes.v1_8_3.title",
    changes: [
      { type: "perf", textKey: "releaseNotes.v1_8_3.changes.0" },
      { type: "ui", textKey: "releaseNotes.v1_8_3.changes.1" },
      { type: "perf", textKey: "releaseNotes.v1_8_3.changes.2" },
    ],
  },
  {
    version: "1.8.2",
    date: "2026-08-20",
    titleKey: "releaseNotes.v1_8_2.title",
    changes: [
      { type: "ui", textKey: "releaseNotes.v1_8_2.changes.0" },
      { type: "perf", textKey: "releaseNotes.v1_8_2.changes.1" },
    ],
  },
]
