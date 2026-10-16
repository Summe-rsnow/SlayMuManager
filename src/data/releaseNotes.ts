export interface ReleaseNoteConfig {
  version: string
  date: string
  tagKey?: string
  titleKey: string
  highlightsKeys?: string[]
  changes: {
    type: "feat" | "perf" | "ui" | "fix" | "opt"
    textKey: string
  }[]
}

export const releaseNotesConfig: ReleaseNoteConfig[] = [
  {
    version: "1.9.1",
    date: "2026-10-16",
    titleKey: "releaseNotes.v1_9_1.title",
    highlightsKeys: [
      "releaseNotes.v1_9_1.highlights.0",
      "releaseNotes.v1_9_1.highlights.1",
    ],
    changes: [
      { type: "ui", textKey: "releaseNotes.v1_9_1.changes.0" },
      { type: "feat", textKey: "releaseNotes.v1_9_1.changes.1" },
      { type: "ui", textKey: "releaseNotes.v1_9_1.changes.2" },
      { type: "fix", textKey: "releaseNotes.v1_9_1.changes.3" },
    ],
  },
  {
    version: "1.9.0",
    date: "2026-10-16",
    titleKey: "releaseNotes.v1_9_0.title",
    highlightsKeys: [
      "releaseNotes.v1_9_0.highlights.0",
      "releaseNotes.v1_9_0.highlights.1",
    ],
    changes: [
      { type: "feat", textKey: "releaseNotes.v1_9_0.changes.0" },
      { type: "feat", textKey: "releaseNotes.v1_9_0.changes.1" },
      { type: "opt", textKey: "releaseNotes.v1_9_0.changes.2" },
      { type: "feat", textKey: "releaseNotes.v1_9_0.changes.3" },
    ],
  },
  {
    version: "1.8.7",
    date: "2026-10-16",
    titleKey: "releaseNotes.v1_8_7.title",
    highlightsKeys: [
      "releaseNotes.v1_8_7.highlights.0",
      "releaseNotes.v1_8_7.highlights.1",
    ],
    changes: [
      { type: "feat", textKey: "releaseNotes.v1_8_7.changes.0" },
      { type: "perf", textKey: "releaseNotes.v1_8_7.changes.1" },
      { type: "perf", textKey: "releaseNotes.v1_8_7.changes.2" },
      { type: "opt", textKey: "releaseNotes.v1_8_7.changes.3" },
    ],
  },
  {
    version: "1.8.6",
    date: "2026-10-06",
    titleKey: "releaseNotes.v1_8_6.title",
    highlightsKeys: [
      "releaseNotes.v1_8_6.highlights.0",
      "releaseNotes.v1_8_6.highlights.1",
    ],
    changes: [
      { type: "feat", textKey: "releaseNotes.v1_8_6.changes.0" },
      { type: "fix", textKey: "releaseNotes.v1_8_6.changes.1" },
      { type: "feat", textKey: "releaseNotes.v1_8_6.changes.2" },
      { type: "perf", textKey: "releaseNotes.v1_8_6.changes.3" },
    ],
  },
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
