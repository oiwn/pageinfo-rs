use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

const SKILL_CONTENT: &str = include_str!("../skills/pginf.md");

const MARKER: &str = "installed-by: pginf";
const META_FILENAME: &str = "pginf.meta";
const SKILL_DIR_NAME: &str = "pginf";
const SKILL_FILE_NAME: &str = "SKILL.md";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn agents_skills_dir(base: &Path) -> PathBuf {
    base.join(".agents").join("skills").join(SKILL_DIR_NAME)
}

fn skill_file_path(base: &Path) -> PathBuf {
    agents_skills_dir(base).join(SKILL_FILE_NAME)
}

fn is_our_install(path: &Path) -> bool {
    fs::read_to_string(path).is_ok_and(|content| content.contains(MARKER))
}

/// Install the skill globally (`~/.agents/skills/pginf/`) or, with `local`,
/// into `.agents/skills/pginf/` relative to the CWD.
pub fn install(local: bool) -> Result<String, String> {
    let base = if local {
        std::env::current_dir().map_err(|e| format!("Cannot get CWD: {e}"))?
    } else {
        dirs::home_dir()
            .ok_or_else(|| "Cannot determine home directory".to_string())?
    };
    install_to(&base)
}

fn install_to(base: &Path) -> Result<String, String> {
    let dir = agents_skills_dir(base);
    let target = skill_file_path(base);

    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create {}: {e}", dir.display()))?;

    if target.exists() && !is_our_install(&target) {
        return Ok(format!(
            "Skipped: {} exists but was not installed by pginf (no '{}' marker). Remove it manually to install.",
            target.display(),
            MARKER
        ));
    }

    let updating = target.exists();
    fs::write(&target, SKILL_CONTENT)
        .map_err(|e| format!("Failed to write {}: {e}", target.display()))?;
    write_manifest(&dir)?;

    let action = if updating { "Updated" } else { "Installed" };
    Ok(format!("{action} skill: {}", target.display()))
}

/// Report install state for the global and local skill targets.
pub fn check() -> Result<String, String> {
    let home = dirs::home_dir()
        .ok_or_else(|| "Cannot determine home directory".to_string())?;
    let cwd =
        std::env::current_dir().map_err(|e| format!("Cannot get CWD: {e}"))?;

    let mut out = String::new();
    for (label, target) in [
        (
            "Global",
            home.join(".agents").join("skills").join(SKILL_DIR_NAME),
        ),
        (
            "Local",
            cwd.join(".agents").join("skills").join(SKILL_DIR_NAME),
        ),
    ] {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&format_target(label, &target));
        out.push('\n');
        let report = check_at(&target, CURRENT_VERSION)?;
        if !report.installed {
            out.push_str("  not installed");
            continue;
        }
        let lines: Vec<String> = report
            .files
            .iter()
            .map(|f| format!("  {:<16} {}", f.path, render_state(f)))
            .collect();
        out.push_str(&lines.join("\n"));
    }
    Ok(out)
}

/// Per-file classification of one install target.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FileState {
    UpToDate,
    Stale,
    LocallyModified,
    NewerThanCurrent,
    UnknownOrigin,
    Missing,
}

#[derive(Debug)]
pub struct FileReport {
    pub path: String,
    pub state: FileState,
    pub installed_by: Option<String>,
}

#[derive(Debug)]
pub struct TargetReport {
    pub installed: bool,
    pub files: Vec<FileReport>,
}

fn check_at(target: &Path, current_version: &str) -> Result<TargetReport, String> {
    let manifest_version = read_manifest_version(target);
    let installed = target.join(SKILL_FILE_NAME).exists();

    let path = target.join(SKILL_FILE_NAME);
    let state = if !path.exists() {
        FileState::Missing
    } else {
        let installed_content = fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        if installed_content == SKILL_CONTENT {
            FileState::UpToDate
        } else {
            match &manifest_version {
                None => FileState::UnknownOrigin,
                Some(v) => match cmp_version(v, current_version) {
                    Ordering::Less => FileState::Stale,
                    Ordering::Equal => FileState::LocallyModified,
                    Ordering::Greater => FileState::NewerThanCurrent,
                },
            }
        }
    };

    Ok(TargetReport {
        installed,
        files: vec![FileReport {
            path: SKILL_FILE_NAME.to_string(),
            state,
            installed_by: manifest_version,
        }],
    })
}

fn write_manifest(target: &Path) -> Result<(), String> {
    fs::write(
        target.join(META_FILENAME),
        format!("pginf_version={CURRENT_VERSION}\n"),
    )
    .map_err(|e| format!("Failed to write manifest: {e}"))
}

fn read_manifest_version(target: &Path) -> Option<String> {
    let content = fs::read_to_string(target.join(META_FILENAME)).ok()?;
    content
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("pginf_version="))
        .map(str::trim)
        .find(|v| !v.is_empty())
        .map(str::to_string)
}

/// Compare two `X.Y.Z` (numeric) version strings component-wise, padding missing
/// components with 0. Non-numeric components are dropped.
fn cmp_version(a: &str, b: &str) -> Ordering {
    let pa = parse_parts(a);
    let pb = parse_parts(b);
    let len = pa.len().max(pb.len());
    for i in 0..len {
        let x = pa.get(i).copied().unwrap_or(0);
        let y = pb.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            Ordering::Equal => continue,
            ord => return ord,
        }
    }
    Ordering::Equal
}

fn parse_parts(s: &str) -> Vec<u64> {
    s.split('.').filter_map(|p| p.parse::<u64>().ok()).collect()
}

fn format_target(label: &str, target: &Path) -> String {
    match label {
        "Global" => {
            let home = dirs::home_dir().unwrap_or_default();
            let rest = target.strip_prefix(&home).unwrap_or(target);
            format!("Global (~/{})", rest.display())
        }
        _ => format!("Local ({})", target.display()),
    }
}

fn render_state(f: &FileReport) -> String {
    match f.state {
        FileState::UpToDate => match &f.installed_by {
            Some(v) => format!("up to date (installed by pginf {v})"),
            None => "up to date".to_string(),
        },
        FileState::Stale => format!(
            "STALE — installed by pginf {}, current is {CURRENT_VERSION}. Reinstall: `pginf skill install`.",
            f.installed_by.as_deref().unwrap_or("?")
        ),
        FileState::LocallyModified => {
            "LOCALLY MODIFIED — differs from bundled. Reinstall will overwrite."
                .to_string()
        }
        FileState::NewerThanCurrent => format!(
            "differs — installed by newer pginf {}, current is {CURRENT_VERSION}.",
            f.installed_by.as_deref().unwrap_or("?")
        ),
        FileState::UnknownOrigin => {
            "UNKNOWN ORIGIN (no version stamp). Reinstall: `pginf skill install`."
                .to_string()
        }
        FileState::Missing => "missing".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_is_in_embedded_content() {
        assert!(
            SKILL_CONTENT.contains(MARKER),
            "Embedded skill content must contain the marker '{MARKER}'"
        );
    }

    #[test]
    fn skill_content_is_not_empty() {
        assert!(!SKILL_CONTENT.is_empty());
    }

    #[test]
    fn agents_skills_dir_path() {
        let base = std::path::Path::new("/tmp/project");
        let dir = agents_skills_dir(base);
        assert_eq!(
            dir,
            std::path::PathBuf::from("/tmp/project/.agents/skills/pginf")
        );
    }

    #[test]
    fn skill_file_path_resolves() {
        let base = std::path::Path::new("/tmp/project");
        let file = skill_file_path(base);
        assert_eq!(
            file,
            std::path::PathBuf::from("/tmp/project/.agents/skills/pginf/SKILL.md")
        );
    }

    #[test]
    fn is_our_install_detects_marker() {
        let dir = std::env::temp_dir()
            .join(format!("pginf-skill-test-{}", std::process::id()));
        let file = dir.join("SKILL.md");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&file, "---\ninstalled-by: pginf\n---\n").unwrap();
        assert!(is_our_install(&file));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn is_our_install_rejects_missing_marker() {
        let dir = std::env::temp_dir()
            .join(format!("pginf-skill-test-nomarker-{}", std::process::id()));
        let file = dir.join("SKILL.md");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&file, "---\nname: pginf\n---\n").unwrap();
        assert!(!is_our_install(&file));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn is_our_install_rejects_missing_file() {
        assert!(!is_our_install(std::path::Path::new(
            "/nonexistent/SKILL.md"
        )));
    }

    #[test]
    fn install_to_creates_dirs_and_writes_file() {
        let dir = std::env::temp_dir()
            .join(format!("pginf-skill-install-test-{}", std::process::id()));
        let result = install_to(&dir).unwrap();
        assert!(result.starts_with("Installed skill:"));
        assert!(skill_file_path(&dir).exists());
        assert_eq!(
            fs::read_to_string(skill_file_path(&dir)).unwrap(),
            SKILL_CONTENT
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn install_writes_manifest_with_version() {
        let dir = std::env::temp_dir()
            .join(format!("pginf-skill-manifest-test-{}", std::process::id()));
        install_to(&dir).unwrap();

        let meta = fs::read_to_string(agents_skills_dir(&dir).join(META_FILENAME))
            .unwrap();
        assert!(meta.contains("pginf_version="));
        assert_eq!(
            read_manifest_version(&agents_skills_dir(&dir)).as_deref(),
            Some(CURRENT_VERSION)
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn install_to_updates_existing_our_install() {
        let dir = std::env::temp_dir()
            .join(format!("pginf-skill-update-test-{}", std::process::id()));
        let target = skill_file_path(&dir);
        fs::create_dir_all(agents_skills_dir(&dir)).unwrap();
        fs::write(&target, format!("---\n{MARKER}\n---\nold content")).unwrap();

        let result = install_to(&dir).unwrap();
        assert!(result.starts_with("Updated skill:"));
        assert_eq!(fs::read_to_string(&target).unwrap(), SKILL_CONTENT);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn install_to_skips_foreign_file() {
        let dir = std::env::temp_dir()
            .join(format!("pginf-skill-skip-test-{}", std::process::id()));
        let target = skill_file_path(&dir);
        fs::create_dir_all(agents_skills_dir(&dir)).unwrap();
        fs::write(&target, "manual content").unwrap();

        let result = install_to(&dir).unwrap();
        assert!(result.contains("Skipped:"));
        assert_eq!(fs::read_to_string(&target).unwrap(), "manual content");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn check_up_to_date_after_install() {
        let dir = std::env::temp_dir()
            .join(format!("pginf-skill-check-ok-test-{}", std::process::id()));
        install_to(&dir).unwrap();
        let skill_dir = agents_skills_dir(&dir);

        let report = check_at(&skill_dir, CURRENT_VERSION).unwrap();
        assert!(report.installed);
        for f in &report.files {
            assert_eq!(f.state, FileState::UpToDate, "{}: {:?}", f.path, f.state);
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn check_stale() {
        let dir = std::env::temp_dir().join(format!(
            "pginf-skill-check-stale-test-{}",
            std::process::id()
        ));
        install_to(&dir).unwrap();
        let skill_dir = agents_skills_dir(&dir);
        // simulate an older install: older content + older version stamp
        fs::write(skill_file_path(&dir), "# old skill\n").unwrap();
        fs::write(skill_dir.join(META_FILENAME), "pginf_version=0.1.0\n").unwrap();

        let report = check_at(&skill_dir, CURRENT_VERSION).unwrap();
        assert_eq!(report.files[0].state, FileState::Stale);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn check_locally_modified() {
        let dir = std::env::temp_dir()
            .join(format!("pginf-skill-check-mod-test-{}", std::process::id()));
        install_to(&dir).unwrap();
        let skill_dir = agents_skills_dir(&dir);
        fs::write(skill_file_path(&dir), "# tampered\n").unwrap();
        // manifest still carries CURRENT_VERSION

        let report = check_at(&skill_dir, CURRENT_VERSION).unwrap();
        assert_eq!(report.files[0].state, FileState::LocallyModified);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn check_unknown_origin_without_manifest() {
        let dir = std::env::temp_dir().join(format!(
            "pginf-skill-check-unknown-test-{}",
            std::process::id()
        ));
        install_to(&dir).unwrap();
        let skill_dir = agents_skills_dir(&dir);
        fs::remove_file(skill_dir.join(META_FILENAME)).unwrap();
        fs::write(skill_file_path(&dir), "# tampered\n").unwrap();

        let report = check_at(&skill_dir, CURRENT_VERSION).unwrap();
        assert_eq!(report.files[0].state, FileState::UnknownOrigin);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn check_not_installed() {
        let dir = std::env::temp_dir().join(format!(
            "pginf-skill-check-none-test-{}",
            std::process::id()
        ));

        let report = check_at(&dir, CURRENT_VERSION).unwrap();
        assert!(!report.installed);
        for f in &report.files {
            assert_eq!(f.state, FileState::Missing, "{}: {:?}", f.path, f.state);
        }
    }

    #[test]
    fn cmp_version_orders_correctly() {
        assert_eq!(cmp_version("0.1.0", "0.2.0"), Ordering::Less);
        assert_eq!(cmp_version("0.2.0", "0.2.0"), Ordering::Equal);
        assert_eq!(cmp_version("0.3.0", "0.2.0"), Ordering::Greater);
        assert_eq!(cmp_version("1.0.0", "0.99.99"), Ordering::Greater);
        assert_eq!(cmp_version("0.2", "0.2.0"), Ordering::Equal); // missing parts padded
        assert_eq!(cmp_version("0.10.0", "0.9.0"), Ordering::Greater); // numeric, not lexical
    }
}
