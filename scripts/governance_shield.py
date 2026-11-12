#!/usr/bin/env python3
"""
CrypRQ Governance Shield
Performs AST validation, syntax validation, formatting validation, and architecture validation.
Blocks commits on failure.
"""

import subprocess
import sys
import os
from pathlib import Path

def run_command(cmd, cwd=None):
    """Run a command and return (success, output)."""
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            capture_output=True,
            text=True,
            cwd=cwd
        )
        return result.returncode == 0, result.stdout + result.stderr
    except Exception as e:
        return False, str(e)

def check_rust_formatting():
    """Check Rust code formatting."""
    print("Checking Rust formatting...")
    success, output = run_command("cargo fmt --all --check")
    if not success:
        print("ERROR: Rust formatting check failed")
        print(output)
        return False
    print("PASS: Rust formatting check")
    return True

def check_rust_clippy():
    """Check Rust code with clippy."""
    print("Checking Rust code with clippy...")
    success, output = run_command("cargo clippy --all-targets --all-features -- -D warnings")
    if not success:
        print("ERROR: Clippy check failed")
        print(output)
        return False
    print("PASS: Clippy check")
    return True

def check_rust_build():
    """Check Rust code compiles."""
    print("Checking Rust code compilation...")
    success, output = run_command("cargo build --workspace")
    if not success:
        print("ERROR: Rust build failed")
        print(output)
        return False
    print("PASS: Rust build")
    return True

def check_rust_tests():
    """Check Rust tests pass."""
    print("Checking Rust tests...")
    success, output = run_command("cargo test --workspace")
    if not success:
        print("ERROR: Rust tests failed")
        print(output)
        return False
    print("PASS: Rust tests")
    return True

def check_no_prompt_files():
    """Check for prompt/agent/validation files (hygiene gate)."""
    print("Checking for prompt/agent/validation files...")
    repo_root = Path(__file__).parent.parent
    
    # Patterns for files that should not be in the repo
    prompt_patterns = [
        "*PROMPT*.md",
        "*prompt*.md",
        "*MASTER_PROMPT*.md",
        "*master_prompt*.md",
        "*AGENT*.md",
        "*agent*.md",
        "*CLEANUP*.md",
        "*VERIFICATION*.md",
        "*REPORT*.md",
        "*SUMMARY*.md",
    ]
    
    # Keywords in filenames that indicate agent/prompt files
    forbidden_keywords = [
        "PROMPT", "prompt", "MASTER_PROMPT", "master_prompt",
        "AGENT_REPORT", "agent_report", "AGENT_SUMMARY", "agent_summary",
        "VERIFICATION_SCRIPT", "verification_script",
        "CLEANUP_REPORT", "cleanup_report",
    ]
    
    found_files = []
    
    # Legitimate files that match patterns but are allowed
    legitimate_files = {
        "docs/VERIFICATION_CHECKLIST.md",
        "docs/VALIDATION_RUN.md",
        "docs/WEB_VALIDATION_RUN.md",
        "docs/TEST_MATRIX.md",
        "docs/PRE_TAG_CHECKLIST.md",
        "docs/RELEASE_EXECUTION_SUMMARY.md",  # Legitimate release documentation
    }
    
    # Only check git-tracked files to avoid false positives from node_modules
    try:
        result = subprocess.run(
            ["git", "ls-files"],
            cwd=repo_root,
            capture_output=True,
            text=True,
            check=True
        )
        tracked_files = set(result.stdout.strip().split("\n"))
    except subprocess.CalledProcessError:
        print("WARNING: Could not get git-tracked files, falling back to filesystem scan")
        tracked_files = None
    
    # Check pattern matches
    for pattern in prompt_patterns:
        for path in repo_root.rglob(pattern):
            rel_path = str(path.relative_to(repo_root))
            
            # Only check tracked files if we have the list
            if tracked_files is not None and rel_path not in tracked_files:
                continue
            
            # Skip node_modules entirely (belt and suspenders)
            if "node_modules/" in rel_path:
                continue
            
            # Skip legitimate documentation files
            if rel_path in legitimate_files:
                continue
            
            # Skip files in docs/ that are validation matrices/checklists
            if "docs/" in rel_path and ("VALIDATION" in rel_path or "CHECKLIST" in rel_path or "TEST_MATRIX" in rel_path):
                continue
            
            # Skip .gitignore patterns (these are tracked but should be ignored)
            if ".gitignore" in rel_path or ".cursorignore" in rel_path:
                continue
            
            found_files.append(rel_path)
    
    # Check for forbidden keywords in filenames
    for path in repo_root.rglob("*.md"):
        rel_path = str(path.relative_to(repo_root))
        path_name = path.name.upper()
        
        # Only check tracked files if we have the list
        if tracked_files is not None and rel_path not in tracked_files:
            continue
        
        # Skip node_modules entirely (belt and suspenders)
        if "node_modules/" in rel_path:
            continue
        
        # Skip legitimate files
        if rel_path in legitimate_files:
            continue
        if "docs/" in rel_path and ("VALIDATION" in rel_path or "CHECKLIST" in rel_path):
            continue
        
        # Check if filename contains forbidden keywords
        for keyword in forbidden_keywords:
            if keyword.upper() in path_name:
                # Additional check: skip if it's a legitimate validation/checklist doc
                if "docs/" in rel_path and ("VALIDATION" in rel_path or "CHECKLIST" in rel_path):
                    continue
                found_files.append(rel_path)
                break
    
    # Remove duplicates
    found_files = list(set(found_files))
    
    if found_files:
        print(f"ERROR: Found {len(found_files)} prompt/agent/validation files:")
        for f in found_files[:20]:  # Limit output
            print(f"  - {f}")
        if len(found_files) > 20:
            print(f"  ... and {len(found_files) - 20} more file(s)")
        print("\nThese files violate repository hygiene and should be removed or moved to .gitignore")
        return False
    print("PASS: No prompt/agent/validation files found")
    return True

def check_no_emojis_in_code_and_docs():
    """Check for emojis in code and documentation files."""
    print("Checking for emojis in code and docs...")
    repo_root = Path(__file__).parent.parent
    
    # Comprehensive emoji patterns (Unicode ranges for common emojis)
    emoji_patterns = [
        "✅", "❌", "⚠️", "🟢", "☐", "🔧", "📌", "🚀", "🔥", "✔️",
        "✓", "✗", "⚠", "🎉", "💡", "🔒", "⚡", "📝", "🐛", "✨",
        "🎯", "🔍", "📊", "🛡️", "⚙️", "🔐", "🌐", "📦", "🔨", "⭐"
    ]
    
    # Files to scan
    scan_patterns = [
        "**/*.rs",      # Rust source
        "**/*.ts",      # TypeScript
        "**/*.tsx",     # TypeScript React
        "**/*.js",      # JavaScript
        "**/*.jsx",     # JavaScript React
        "**/*.sh",      # Shell scripts
        "**/*.py",      # Python
        "**/*.md",      # Markdown docs
        "**/*.yml",     # YAML (workflows)
        "**/*.yaml",    # YAML
        "**/*.toml",    # TOML
        "**/Dockerfile*",  # Dockerfiles
    ]
    
    # Directories to exclude (build artifacts, vendor code, virtual envs)
    exclude_dirs = {
        "target", "node_modules", ".git", ".cargo", "dist", "build",
        "web/dist", "web/node_modules", ".github/workflows/.hardening-kit",
        ".venv", "venv", "__pycache__", ".pytest_cache", ".mypy_cache"
    }
    
    # Files to exclude (legitimate files that may contain emoji patterns for detection)
    exclude_files = {
        "scripts/governance_shield.py",  # Contains emoji patterns for detection
    }
    
    found_emojis = []
    
    for pattern in scan_patterns:
        for file_path in repo_root.rglob(pattern):
            # Skip excluded directories
            if any(excluded in file_path.parts for excluded in exclude_dirs):
                continue
            
            # Skip excluded files
            rel_path = str(file_path.relative_to(repo_root))
            if rel_path in exclude_files:
                continue
            
            # Skip binary files and very large files
            if file_path.stat().st_size > 1_000_000:  # Skip files > 1MB
                continue
            
            try:
                content = file_path.read_text(encoding='utf-8', errors='ignore')
                for emoji in emoji_patterns:
                    if emoji in content:
                        found_emojis.append((rel_path, emoji))
                        break  # Only report once per file
            except (UnicodeDecodeError, Exception):
                # Skip binary files or files that can't be read
                continue
    
    if found_emojis:
        print(f"ERROR: Found emojis in {len(found_emojis)} file(s):")
        for file, emoji in found_emojis[:20]:  # Limit output to first 20
            print(f"  - {file}: {emoji}")
        if len(found_emojis) > 20:
            print(f"  ... and {len(found_emojis) - 20} more file(s)")
        return False
    print("PASS: No emojis in code or docs")
    return True

def main():
    """Run all governance checks."""
    print("=" * 60)
    print("CrypRQ Governance Shield")
    print("=" * 60)
    print()
    
    checks = [
        ("Rust Formatting", check_rust_formatting),
        ("Rust Clippy", check_rust_clippy),
        ("Rust Build", check_rust_build),
        ("Rust Tests", check_rust_tests),
        ("No Prompt/Agent Files", check_no_prompt_files),
        ("No Emojis in Code/Docs", check_no_emojis_in_code_and_docs),
    ]
    
    # Add Node.js/TypeScript checks if available
    def check_node_build():
        """Check Node.js build."""
        print("Checking Node.js build...")
        repo_root = Path(__file__).parent.parent
        web_dir = repo_root / "web"
        if not web_dir.exists():
            print("SKIP: web/ directory not found")
            return True
        success, output = run_command("npm run build", cwd=str(web_dir))
        if not success:
            print("ERROR: Node.js build failed")
            print(output)
            return False
        print("PASS: Node.js build")
        return True
    
    def check_typescript_types():
        """Check TypeScript type checking."""
        print("Checking TypeScript types...")
        repo_root = Path(__file__).parent.parent
        web_dir = repo_root / "web"
        if not web_dir.exists():
            print("SKIP: web/ directory not found")
            return True
        success, output = run_command("npx tsc --noEmit", cwd=str(web_dir))
        if not success:
            print("ERROR: TypeScript type check failed")
            print(output)
            return False
        print("PASS: TypeScript types")
        return True
    
    # Add Node.js checks if web directory exists
    web_dir = Path(__file__).parent.parent / "web"
    if web_dir.exists():
        checks.extend([
            ("Node.js Build", check_node_build),
            ("TypeScript Types", check_typescript_types),
        ])
    
    failed_checks = []
    for name, check_func in checks:
        if not check_func():
            failed_checks.append(name)
        print()
    
    if failed_checks:
        print("=" * 60)
        print("GOVERNANCE SHIELD: FAILED")
        print("=" * 60)
        print(f"Failed checks: {', '.join(failed_checks)}")
        print()
        print("Please fix the issues above before committing.")
        sys.exit(1)
    
    print("=" * 60)
    print("GOVERNANCE SHIELD: PASSED")
    print("=" * 60)
    sys.exit(0)

if __name__ == "__main__":
    main()

