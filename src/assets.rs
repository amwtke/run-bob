//! Single source of truth for the harness assets.
//!
//! `init` installs from this registry; `status` verifies against it.
//! Adding a new template means adding one entry here — `init` will install
//! it and `status` will check it, with no further bookkeeping.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Skill,
    HarnessDoc,
    ArchUnit,
    SharedJava,
}

impl Category {
    pub fn install_header(self) -> &'static str {
        match self {
            Category::Skill => "Installing skills...",
            Category::HarnessDoc => "Installing harness documents...",
            Category::ArchUnit => "Installing ArchUnit guards...",
            Category::SharedJava => "Installing shared Java skeletons...",
        }
    }

    pub fn status_header(self) -> &'static str {
        match self {
            Category::Skill => "Skills",
            Category::HarnessDoc => "Harness documents",
            Category::ArchUnit => "ArchUnit guards",
            Category::SharedJava => "Shared Java skeletons",
        }
    }
}

pub struct Asset {
    /// Path segments relative to the target directory.
    pub rel_path: &'static [&'static str],
    /// File content baked at compile time via `include_str!`.
    pub content: &'static str,
    pub category: Category,
    /// True if the asset is installed even in `--minimal` mode.
    pub included_in_minimal: bool,
    /// True if `run-bob upgrade` may overwrite this file when its content
    /// drifts from the embedded version. User-owned files (CLAUDE.md,
    /// ARCHITECTURE.md, CleanArchitectureTest.java) MUST be false.
    pub upgrade_safe: bool,
}

impl Asset {
    pub fn display(&self) -> String {
        self.rel_path.join("/")
    }
}

pub struct HarnessDir {
    pub rel_path: &'static [&'static str],
    pub note: &'static str,
}

impl HarnessDir {
    pub fn display(&self) -> String {
        format!("{}/", self.rel_path.join("/"))
    }
}

pub const HARNESS_ASSETS: &[Asset] = &[
    // --- Skills (installed even in --minimal) ---
    Asset {
        rel_path: &[".claude", "skills", "bob-identify", "SKILL.md"],
        content: include_str!("templates/skills/bob-identify.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[".claude", "skills", "bob-onion", "SKILL.md"],
        content: include_str!("templates/skills/bob-onion.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[".claude", "skills", "bob-spec", "SKILL.md"],
        content: include_str!("templates/skills/bob-spec.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[".claude", "skills", "bob-survey", "SKILL.md"],
        content: include_str!("templates/skills/bob-survey.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[".claude", "skills", "bob-stories", "SKILL.md"],
        content: include_str!("templates/skills/bob-stories.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[".claude", "skills", "bob-nfr", "SKILL.md"],
        content: include_str!("templates/skills/bob-nfr.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[".claude", "skills", "bob-compliance", "SKILL.md"],
        content: include_str!("templates/skills/bob-compliance.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[".claude", "skills", "bob-model", "SKILL.md"],
        content: include_str!("templates/skills/bob-model.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    // --- Harness documents (skipped in --minimal) ---
    Asset {
        rel_path: &["CLAUDE.md"],
        content: include_str!("templates/root/CLAUDE.md"),
        category: Category::HarnessDoc,
        included_in_minimal: false,
        upgrade_safe: false,
    },
    Asset {
        rel_path: &["ARCHITECTURE.md"],
        content: include_str!("templates/root/ARCHITECTURE.md"),
        category: Category::HarnessDoc,
        included_in_minimal: false,
        upgrade_safe: false,
    },
    Asset {
        rel_path: &["README-RUN-BOB.md"],
        content: include_str!("templates/root/README-RUN-BOB.md"),
        category: Category::HarnessDoc,
        included_in_minimal: false,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &["docs", "compliance", "README.md"],
        content: include_str!("templates/root/compliance-README.md"),
        category: Category::HarnessDoc,
        included_in_minimal: false,
        upgrade_safe: true,
    },
    // --- ArchUnit guard ---
    Asset {
        rel_path: &[
            "src", "test", "java", "architecture", "CleanArchitectureTest.java",
        ],
        content: include_str!("templates/root/CleanArchitectureTest.java"),
        category: Category::ArchUnit,
        included_in_minimal: false,
        upgrade_safe: false,
    },
    // --- Shared Java skeletons ---
    Asset {
        rel_path: &[
            "src", "main", "java", "com", "example", "shared", "usecase", "UseCase.java",
        ],
        content: include_str!("templates/root/UseCase.java"),
        category: Category::SharedJava,
        included_in_minimal: false,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[
            "src", "main", "java", "com", "example", "shared", "framework",
            "transaction", "TransactionalUseCaseDecorator.java",
        ],
        content: include_str!("templates/root/TransactionalUseCaseDecorator.java"),
        category: Category::SharedJava,
        included_in_minimal: false,
        upgrade_safe: true,
    },
];

pub const HARNESS_DIRS: &[HarnessDir] = &[
    HarnessDir {
        rel_path: &["docs", "bob"],
        note: "(identify & onion intermediate notes)",
    },
    HarnessDir {
        rel_path: &["docs", "specs"],
        note: "(bob-spec outputs → Superpowers inputs)",
    },
    HarnessDir {
        rel_path: &["docs", "compliance", "sources"],
        note: "(用户合规规约原始文件 — PDF/docx/md/txt;空目录 = 无合规要求)",
    },
];
