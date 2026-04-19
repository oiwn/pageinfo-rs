use std::fs;
use std::path::PathBuf;

const SKILL_CONTENT: &str = include_str!("../skills/pginf.md");

const MARKER: &str = "installed-by: pginf";
const SKILL_DIR_NAME: &str = "pginf";
const SKILL_FILE_NAME: &str = "SKILL.md";

fn agents_skills_dir(base: &std::path::Path) -> PathBuf {
    base.join(".agents").join("skills").join(SKILL_DIR_NAME)
}

fn skill_file_path(base: &std::path::Path) -> PathBuf {
    agents_skills_dir(base).join(SKILL_FILE_NAME)
}

fn is_our_install(path: &std::path::Path) -> bool {
    fs::read_to_string(path).is_ok_and(|content| content.contains(MARKER))
}

pub fn install_local() -> Result<String, String> {
    let base =
        std::env::current_dir().map_err(|e| format!("Cannot get CWD: {e}"))?;
    install_to(&base)
}

pub fn install_global() -> Result<String, String> {
    let home = dirs::home_dir()
        .ok_or_else(|| "Cannot determine home directory".to_string())?;
    install_to(&home)
}

fn install_to(base: &std::path::Path) -> Result<String, String> {
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

    let action = if updating { "Updated" } else { "Installed" };
    Ok(format!("{action} skill: {}", target.display()))
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
}
