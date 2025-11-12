# GitHub Wiki Setup Guide

This guide explains how to set up the GitHub wiki for the CrypRQ repository.

## Prerequisites

1. Repository admin access
2. Git installed locally
3. GitHub wiki enabled in repository settings

## Step 1: Enable Wiki in GitHub Settings

1. Go to: https://github.com/codethor0/cryprq/settings
2. Scroll down to the "Features" section
3. Find "Wikis" and check the box to enable it
4. Optionally, restrict editing to repository collaborators only

## Step 2: Clone the Wiki Repository

Once the wiki is enabled, GitHub creates a separate wiki repository. Clone it:

```bash
git clone https://github.com/codethor0/cryprq.wiki.git wiki-temp
```

## Step 3: Copy Wiki Files

The wiki files are located in `.github/wiki/` directory. Copy them to the wiki repository:

```bash
# Copy all markdown files
cp .github/wiki/*.md wiki-temp/
```

Or use the automated script:

```bash
bash scripts/setup-github-wiki.sh
```

## Step 4: Commit and Push

```bash
cd wiki-temp
git add *.md
git commit -m "docs: add wiki pages from repository"
git push origin main
```

## Step 5: Verify

Visit the wiki at: https://github.com/codethor0/cryprq/wiki

You should see:
- Home
- Getting Started
- VPN Setup
- Testing
- Troubleshooting

## Wiki Files Included

The following pages are included:

1. **Home.md** - Overview and quick links
2. **Getting-Started.md** - Installation and quick start guide
3. **VPN-Setup.md** - Complete VPN deployment guide
4. **Testing.md** - Testing information and results
5. **Troubleshooting.md** - Common issues and solutions

## Updating the Wiki

To update the wiki:

1. Make changes to files in `.github/wiki/`
2. Commit changes to the main repository
3. Copy updated files to wiki repository
4. Commit and push to wiki repository

Or use the setup script again:

```bash
bash scripts/setup-github-wiki.sh
```

## Notes

- Wiki files are version controlled in the main repository at `.github/wiki/`
- The GitHub wiki is a separate git repository
- Both should be kept in sync
- Wiki files are emoji-free and production-ready

